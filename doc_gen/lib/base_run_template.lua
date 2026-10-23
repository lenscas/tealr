local etlua = etlua()
---comment
---@param name string
---@param tbl any
function inspect_type(name, tbl)
    if type(tbl) == "table" then
        local looped_once = false
        for k, v in pairs(tbl) do
            looped_once = true
            local typeOf = type(v)
            if typeOf == "table" then
                print(name, " has ", k, "of", typeOf, "with")
                inspect_type(name .. " - " .. k, v)
            else
                print(name, " has ", k, "of", typeOf, "containing", v)
            end
        end
        if not looped_once then
            print(name .. " is empty table")
        end
    else
        print(name, " is a", type(tbl), "containing", tbl)
    end
end

---@generic T
---@param t1 T[] table to be inserted into
---@param t2 T[] table to be inserted
---@return T[]
function concat_array(t1, t2)
    for _, v in ipairs(t2) do
        table.insert(t1, v)
    end
    return t1
end

---@generic T
---@generic U
---@param tbl T[]
---@param mapper function(T):U
---@return U[]
function map(tbl, mapper)
    local result = {}
    for _, v in ipairs(tbl) do
        table.insert(result, mapper(v))
    end
    return result
end

---comment
---@param types Type[]
---@param generics Name[]
---@return table
function get_missing_generics(types, generics)
    local current_generics = {}
    for _, v in ipairs(types) do
        local found_generics = find_generics(v)
        concat_array(current_generics, found_generics)
    end
    current_generics = dedupe_by(
        current_generics,
        function(a)
            return a
        end
    )
    local generics_to_process = {}
    for _, v in ipairs(current_generics) do
        local found = false
        for _, v2 in ipairs(generics) do
            if v == v2 then
                found = true
                break
            end
        end
        if not found then
            table.insert(generics_to_process, v)
        end
    end
    return generics_to_process
end

---@class RenderOptionsOptions
---@field generics Name[]
---@field in_params boolean
---@field is_variadic boolean

---comment
---@param render_in function(string):string
---@param render_html_in? function(string):string
---@param type_appendage? string
---@return RenderOptions
function get_type_renderer(render_in, render_html_in, type_appendage)
    local render = function(a)
        render_in(a)
        return a
    end
    type_appendage = (type_appendage and (type_appendage .. ".")) or ""
    local render_html
    if render_html_in then
        render_html = function(a)
            render_html_in(a)
            return a
        end
    end
    local renderer = {}
    renderer["function"] = function(funcRes, extra)
        local rendered = render("function")
        local generics = extra.generics or {}


        local generics_to_process = get_missing_generics(map(funcRes.params, function(v)
            return v.ty
        end), generics)
        generics_to_process = concat_array(generics_to_process, get_missing_generics(funcRes.returns, generics))
        generics_to_process = dedupe_by(generics_to_process, function(a) return a end)
        generics = concat_array(generics, generics_to_process)
        if #generics_to_process >= 1 then
            rendered = rendered .. render("<")
            for k, v in ipairs(generics_to_process) do
                if k ~= 1 then
                    rendered = rendered .. render(", ")
                end
                if type(v) == "table" and type(v[0]) == "string" then
                    rendered = rendered .. render(v[0])
                else
                    rendered = rendered .. type_to_string(v, renderer, { generics = generics })
                end
            end
            rendered = rendered .. render(">")
        end
        rendered = rendered .. render("(")
        for k, v in ipairs(funcRes.params) do
            if k > 1 then
                rendered = rendered .. render(" , ")
            end
            local param_name = v.param_name
            --variadic types _always_ have the name "..." when used as a parameter of a function
            if v.ty:IsVariadic() then
                param_name = "..."
            end
            if type(param_name) == "table" then
                param_name = param_name.param0
            end
            if param_name then
                rendered = rendered .. render(param_name)
                rendered = rendered .. render(" : ")
            end
            rendered = rendered .. type_to_string(v.ty, renderer, { in_params = true, generics = generics })
        end
        rendered = rendered .. render(")")
        local endRender = ""
        if #funcRes.returns > 1 then
            rendered = rendered .. render(": ((")
            endRender = "))"
        elseif #funcRes.returns == 1 then
            rendered = rendered .. render(" : ")
            local toReturn = funcRes.returns[1]
            if not toReturn:IsSingle() then
                rendered = rendered .. render("(")
                endRender = ")"
            end
        end
        for k, v in ipairs(funcRes.returns) do
            if k > 1 then
                if v:IsVariadic() then
                    rendered = rendered .. render(" ) , ")
                    endRender = ")"
                else
                    rendered = rendered .. render(" ) , (")
                end
            end
            rendered = rendered .. type_to_string(v, renderer, { in_return = true, generics = generics })
        end
        rendered = rendered .. render(endRender)

        return rendered
    end
    function renderer.map(mapRes, extra)
        local rendered = render("{ ")
        rendered = rendered .. type_to_string(mapRes.key, renderer, { generics = extra.generics })
        rendered = rendered .. render(" : ")
        rendered = rendered .. type_to_string(mapRes.value, renderer, { generics = extra.generics })
        rendered = rendered .. render(" } ")
        return rendered
    end

    renderer["or"] = function(orRes, extra)
        local rendered = ""
        if not extra.is_variadic then
            rendered = rendered .. render("( ")
        end
        for k, v in ipairs(orRes) do
            if k > 1 then
                rendered = rendered .. render(" | ")
            end
            rendered = rendered .. type_to_string(v, renderer, extra)
        end
        if not extra.is_variadic then
            rendered = rendered .. render(" )")
        end
        return rendered
    end
    function renderer.array(arrayRes, extra)
        local rendered = render("{ ")
        rendered = rendered .. type_to_string(arrayRes, renderer, { generics = extra.generics })
        rendered = rendered .. render(" }")
        return rendered
    end

    function renderer.tuple(orRes, extra)
        local rendered = ""
        if not extra.is_variadic then
            rendered = rendered .. render("( ")
        end
        for k, v in ipairs(orRes) do
            if k > 1 then
                rendered = rendered .. render(" , ")
            end
            rendered = rendered .. type_to_string(v, renderer, extra)
        end
        if not extra.is_variadic then
            rendered = rendered .. render(" )")
        end
        return rendered
    end

    function renderer.single(ty, extra)
        local rendered = ""
        local name = ty.name
        if type(name) == "table" then
            name = name.param0
        end
        if ty.kind == "External" then
            if render_html then
                render_html("<a href=\"")
                render(type_to_link(ty))
                render_html("\">")
            end
            rendered = rendered .. render(type_appendage)
        end
        rendered = rendered .. render(name)
        if #ty.generics > 0 then
            rendered = rendered .. render("<")
            for k, v in ipairs(ty.generics) do
                if k > 1 then
                    rendered = rendered .. render(", ")
                end
                rendered = rendered .. type_to_string(v, renderer, { generics = extra.generics })
            end
            rendered = rendered .. render(">")
        end
        if render_html and ty.kind == "External" then
            render_html("</a>")
        end
        return rendered
    end

    function renderer.variadic(varRes, extra)
        if not (
                extra.in_params
                or
                extra.in_return
            ) then
            print(
                "Found a variadic type that is not part of a function signature. THIS iS AN IMPOSSIBLE PLACE FOR A VARIADIC TO SHOW UP!"
            )
        end
        local rendered = type_to_string(varRes, renderer, { generics = extra.generics, is_variadic = true })
        if extra.in_return then
            rendered = rendered .. render("...")
        end
        return rendered
    end

    return renderer
end

---comment
---@param render fun(string):string
---@param render_function fun(to_render: FunctionRepresentation, generics: Type[]):string
---@param render_type fun(type: Type, generics: Type[]):string
---@return fun(member:table, generics: Type[])
function getMacroRenderer(render, render_function, render_type)
    return function(member, generics)
        render_function(member.signature, generics)
        render(" = macroexp(")


        local as_ty = ty.NewFunctionFrom(member.signature)
        local all_generics = generics

        member.signature = as_ty:GetFunctionOrNil() --restore the otherwise consumed userdata
        local size = #member.signature.params
        for k, v in ipairs(member.signature.params) do
            render(v.param_name.param0)
            render(" : ")
            render_type(v.ty, all_generics)
            if k < size then
                render(",")
            end
        end
        render("): ")
        render_type(member.signature.returns[1], all_generics)
        render("\n")
        render(member.expr)
        render("end")
    end
end

---@param check string the haystack
---@param start string the needle being searched for
---@return boolean
function startsWith(check, start)
    return string.sub(check, 1, check.len(start)) == start
end

---@param check string the haystack
---@param ending string the needle being searched for
function endsWith(check, ending)
    return ending == "" or check:sub(- #ending) == ending
end

local template, err = etlua.compile(template)
if not template then print(err) end
return template()
