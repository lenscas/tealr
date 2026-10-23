use std::collections::HashSet;

use tealr::TypeWalker;

use crate::is_type_compatible;

/// Goes over every function, field and inheritance to look for any type that is used but not exported
/// and then prints a warning about it to stderr.
pub fn warn_about_missing_exports(type_defs: &TypeWalker) {
    let mut missing = find_types_missing_export(type_defs);
    missing.sort_by_key(|(x, _)| x.name.to_string());
    let grouped = missing
        .into_iter()
        .fold(Vec::new(), |mut acc, (ty, used_by)| {
            match acc.last_mut() {
                None => {
                    acc.push((ty, vec![used_by]));
                }
                Some((current, users)) if current.name == ty.name => {
                    users.push(used_by);
                }
                Some((_, _)) => {
                    acc.push((ty, vec![used_by]));
                }
            }
            acc
        });
    for (ty, used_by) in grouped {
        match used_by.as_slice() {
            [] => {
                eprintln!(
                    "Missing type from export: {}. But no matching use found? This sounds like a bug.",
                    ty.name
                );
            }
            [x] => {
                eprintln!("Missing type from export: {}. Used by: {}", ty.name, x);
            }
            [x, y @ ..] => {
                eprintln!(
                    "Missing type from export: {}. Used by: {} and {} others",
                    ty.name,
                    x,
                    y.len()
                );
            }
        }
    }
}

/// Finds every type that is used but not exported
///
/// The returned type is the missing type. The string is the place it is used
pub fn find_types_missing_export(walker: &TypeWalker) -> Vec<(tealr::SingleType, String)> {
    let mut missing_types = Vec::new();
    let all_exported_types = walker
        .iter()
        .map(|x| x.type_name().to_owned())
        .filter_map(|v| v.single().map(ToOwned::to_owned))
        .collect::<HashSet<_>>();
    let all_used_types = walker
        .iter()
        .flat_map(|x| {
            if let tealr::TypeGenerator::Record(record_generator) = x {
                let name = tealr::type_to_string(&record_generator.ty, false);
                let name2 = name.clone();
                record_generator
                    .fields
                    .iter()
                    .map(move |x| {
                        (
                            x.ty.clone(),
                            format!("{}.{}", name, String::from_utf8_lossy(&x.name)),
                        )
                    })
                    .chain(extract_all_function_types_from_record_generator(
                        record_generator,
                    ))
                    .chain(record_generator.static_fields.iter().map(|x| {
                        (
                            x.ty.clone(),
                            format!("{}.{}", name2, String::from_utf8_lossy(&x.name)),
                        )
                    }))
                    .collect::<HashSet<_>>()
            } else {
                HashSet::new()
            }
        })
        .flat_map(|(x, belongs_to)| {
            extract_all_singular_types_from_type(&x).map(move |v| (v, belongs_to.clone()))
        })
        .chain(walker.global_instances_off.iter().flat_map(|x| {
            extract_all_singular_types_from_type(&x.ty).map(move |x| {
                let name = format!("global.{}", x.name.clone());
                (x, name)
            })
        }))
        .filter(|(ty, _)| ty.kind == tealr::KindOfType::External)
        .collect::<HashSet<_>>();
    for (ty, belongs_to) in all_used_types {
        if !all_exported_types
            .iter()
            .any(|v| is_type_compatible(v, ty.to_owned()))
        {
            missing_types.push((ty, belongs_to));
        }
    }
    missing_types
}

fn extract_all_singular_types_from_type<'a>(
    ty: &tealr::Type,
) -> impl Iterator<Item = tealr::SingleType> + use<'a> {
    match ty {
        tealr::Type::Function(function_representation) => function_representation
            .params
            .iter()
            .flat_map(|x| extract_all_singular_types_from_type(&x.ty))
            .chain(
                function_representation
                    .returns
                    .iter()
                    .flat_map(extract_all_singular_types_from_type),
            )
            .collect::<HashSet<_>>(),
        tealr::Type::Single(single_type) => {
            std::iter::once(single_type.clone()).collect::<HashSet<_>>()
        }
        tealr::Type::Map(map_representation) => {
            extract_all_singular_types_from_type(&map_representation.key)
                .chain(extract_all_singular_types_from_type(
                    &map_representation.value,
                ))
                .collect::<HashSet<_>>()
        }
        tealr::Type::Tuple(vec) | tealr::Type::Or(vec) => vec
            .iter()
            .flat_map(extract_all_singular_types_from_type)
            .collect::<HashSet<_>>(),
        tealr::Type::Array(x) => extract_all_singular_types_from_type(x).collect::<HashSet<_>>(),
        tealr::Type::Variadic(x) => extract_all_singular_types_from_type(x).collect::<HashSet<_>>(),
    }
    .into_iter()
}

fn extract_all_function_types_from_record_generator(
    record_generator: &tealr::RecordGenerator,
) -> impl Iterator<Item = (tealr::Type, String)> + '_ {
    let name = tealr::type_to_string(&record_generator.ty, false).to_string();
    extract_types_from_exported_functions(&record_generator.methods, name.clone())
}

fn extract_types_from_exported_functions(
    exported_functions: &[tealr::ExportedFunction],
    record_name: String,
) -> impl Iterator<Item = (tealr::Type, String)> + '_ {
    exported_functions
        .iter()
        .flat_map(move |x| extract_types_from_exported_function(x, record_name.clone()))
}

fn extract_types_from_exported_function(
    exported_function: &tealr::ExportedFunction,
    record_name: String,
) -> impl Iterator<Item = (tealr::Type, String)> + '_ {
    let full_name = format!(
        "{}.{}",
        record_name,
        String::from_utf8_lossy(&exported_function.name)
    );
    let full_name2 = full_name.clone();
    exported_function
        .params
        .iter()
        .map(move |x| (x.ty.clone(), full_name.clone()))
        .chain(
            exported_function
                .returns
                .iter()
                .map(move |x| (x.to_owned(), full_name2.clone())),
        )
}
