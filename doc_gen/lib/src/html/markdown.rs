//! Wrapper around [pulldown_cmark], adding support for lua.

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType,
    MetadataBlockKind, Options, Parser, Tag, TagEnd, html,
};
use tealr::{
    ToTypename,
    mlu::{FromToLua, mlua::FromLua},
};

///Exposing the original crate
pub use pulldown_cmark;

#[derive(Clone, Debug, ToTypename, FromToLua)]
///What kind of codeblock it is
pub enum MarkdownCodeBlockKind {
    Indented,
    /// The value contained in the tag describes the language of the code, which may be empty.
    Fenced(String),
}

impl From<CodeBlockKind<'_>> for MarkdownCodeBlockKind {
    fn from(x: CodeBlockKind) -> Self {
        match x {
            CodeBlockKind::Indented => Self::Indented,
            CodeBlockKind::Fenced(x) => Self::Fenced(x.to_string()),
        }
    }
}

impl From<MarkdownCodeBlockKind> for CodeBlockKind<'static> {
    fn from(x: MarkdownCodeBlockKind) -> Self {
        match x {
            MarkdownCodeBlockKind::Indented => Self::Indented,
            MarkdownCodeBlockKind::Fenced(x) => Self::Fenced(x.into()),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, ToTypename, FromToLua)]
pub enum MarkdownHeadingLevel {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<HeadingLevel> for MarkdownHeadingLevel {
    fn from(x: HeadingLevel) -> Self {
        match x {
            HeadingLevel::H1 => Self::H1,
            HeadingLevel::H2 => Self::H2,
            HeadingLevel::H3 => Self::H3,
            HeadingLevel::H4 => Self::H4,
            HeadingLevel::H5 => Self::H5,
            HeadingLevel::H6 => Self::H6,
        }
    }
}
impl From<MarkdownHeadingLevel> for HeadingLevel {
    fn from(x: MarkdownHeadingLevel) -> Self {
        match x {
            MarkdownHeadingLevel::H1 => Self::H1,
            MarkdownHeadingLevel::H2 => Self::H2,
            MarkdownHeadingLevel::H3 => Self::H3,
            MarkdownHeadingLevel::H4 => Self::H4,
            MarkdownHeadingLevel::H5 => Self::H5,
            MarkdownHeadingLevel::H6 => Self::H6,
        }
    }
}
/// Text alignment in tables
#[derive(ToTypename, Debug, FromToLua, Clone)]
pub enum MarkdownAlignment {
    /// Default text alignment.
    None,
    Left,
    Center,
    Right,
}
impl From<Alignment> for MarkdownAlignment {
    fn from(x: Alignment) -> Self {
        match x {
            Alignment::None => Self::None,
            Alignment::Left => Self::Left,
            Alignment::Center => Self::Center,
            Alignment::Right => Self::Right,
        }
    }
}

impl From<MarkdownAlignment> for Alignment {
    fn from(x: MarkdownAlignment) -> Self {
        match x {
            MarkdownAlignment::None => Self::None,
            MarkdownAlignment::Left => Self::Left,
            MarkdownAlignment::Center => Self::Center,
            MarkdownAlignment::Right => Self::Right,
        }
    }
}

#[derive(ToTypename, Debug, FromToLua, Clone)]
/// Type specifier for inline links
pub enum MarkdownLinkType {
    /// Inline link like `[foo](bar)`
    Inline,
    /// Reference link like `[foo][bar]`
    Reference,
    /// Reference without destination in the document, but resolved by the broken_link_callback
    ReferenceUnknown,
    /// Collapsed link like `[foo][]`
    Collapsed,
    /// Collapsed link without destination in the document, but resolved by the broken_link_callback
    CollapsedUnknown,
    /// Shortcut link like `[foo]`
    Shortcut,
    /// Shortcut without destination in the document, but resolved by the broken_link_callback
    ShortcutUnknown,
    /// Autolink like `<http://foo.bar/baz>`
    Autolink,
    /// Email address in autolink like `<john@example.org>`
    Email,
}

impl From<LinkType> for MarkdownLinkType {
    fn from(x: LinkType) -> Self {
        match x {
            LinkType::Inline => Self::Inline,
            LinkType::Reference => Self::Reference,
            LinkType::ReferenceUnknown => Self::ReferenceUnknown,
            LinkType::Collapsed => Self::Collapsed,
            LinkType::CollapsedUnknown => Self::CollapsedUnknown,
            LinkType::Shortcut => Self::Shortcut,
            LinkType::ShortcutUnknown => Self::ShortcutUnknown,
            LinkType::Autolink => Self::Autolink,
            LinkType::Email => Self::Email,
        }
    }
}

impl From<MarkdownLinkType> for LinkType {
    fn from(x: MarkdownLinkType) -> Self {
        match x {
            MarkdownLinkType::Inline => Self::Inline,
            MarkdownLinkType::Reference => Self::Reference,
            MarkdownLinkType::ReferenceUnknown => Self::ReferenceUnknown,
            MarkdownLinkType::Collapsed => Self::Collapsed,
            MarkdownLinkType::CollapsedUnknown => Self::CollapsedUnknown,
            MarkdownLinkType::Shortcut => Self::Shortcut,
            MarkdownLinkType::ShortcutUnknown => Self::ShortcutUnknown,
            MarkdownLinkType::Autolink => Self::Autolink,
            MarkdownLinkType::Email => Self::Email,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromToLua, ToTypename)]
pub enum MarkdownBlockQuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl From<BlockQuoteKind> for MarkdownBlockQuoteKind {
    fn from(x: BlockQuoteKind) -> Self {
        match x {
            BlockQuoteKind::Note => Self::Note,
            BlockQuoteKind::Tip => Self::Tip,
            BlockQuoteKind::Important => Self::Important,
            BlockQuoteKind::Warning => Self::Warning,
            BlockQuoteKind::Caution => Self::Caution,
        }
    }
}

impl From<MarkdownBlockQuoteKind> for BlockQuoteKind {
    fn from(x: MarkdownBlockQuoteKind) -> Self {
        match x {
            MarkdownBlockQuoteKind::Note => Self::Note,
            MarkdownBlockQuoteKind::Tip => Self::Tip,
            MarkdownBlockQuoteKind::Important => Self::Important,
            MarkdownBlockQuoteKind::Warning => Self::Warning,
            MarkdownBlockQuoteKind::Caution => Self::Caution,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromToLua, ToTypename)]

pub enum MarkdownMetadataBlockKind {
    YamlStyle,
    PlusesStyle,
}
impl From<MetadataBlockKind> for MarkdownMetadataBlockKind {
    fn from(x: MetadataBlockKind) -> Self {
        match x {
            MetadataBlockKind::YamlStyle => Self::YamlStyle,
            MetadataBlockKind::PlusesStyle => Self::PlusesStyle,
        }
    }
}

impl From<MarkdownMetadataBlockKind> for MetadataBlockKind {
    fn from(x: MarkdownMetadataBlockKind) -> Self {
        match x {
            MarkdownMetadataBlockKind::YamlStyle => Self::YamlStyle,
            MarkdownMetadataBlockKind::PlusesStyle => Self::PlusesStyle,
        }
    }
}

#[derive(Clone, Debug, FromToLua, ToTypename)]
pub struct MarkdownAttribute {
    pub key: String,
    pub value: Option<String>,
}

impl From<(String, Option<String>)> for MarkdownAttribute {
    fn from(x: (String, Option<String>)) -> Self {
        Self {
            key: x.0,
            value: x.1,
        }
    }
}
impl<'a> From<(CowStr<'a>, Option<CowStr<'a>>)> for MarkdownAttribute {
    fn from(x: (CowStr, Option<CowStr>)) -> Self {
        Self {
            key: x.0.to_string(),
            value: x.1.map(|x| x.to_string()),
        }
    }
}

impl From<MarkdownAttribute> for (String, Option<String>) {
    fn from(x: MarkdownAttribute) -> Self {
        (x.key, x.value)
    }
}

impl<'a> From<MarkdownAttribute> for (CowStr<'a>, Option<CowStr<'a>>) {
    fn from(x: MarkdownAttribute) -> Self {
        (x.key.into(), x.value.map(|x| x.into()))
    }
}

#[derive(Clone, Debug, FromToLua, ToTypename)]
/// Tags containing other elements
pub enum MarkdownTag {
    /// A paragraph of text and other inline elements.
    Paragraph,

    /// A heading. The first field indicates the level of the heading,
    /// the second the fragment identifier, and the third the classes.
    Heading(
        MarkdownHeadingLevel,
        Option<String>,
        Vec<String>,
        Vec<MarkdownAttribute>,
    ),

    BlockQuote(Option<MarkdownBlockQuoteKind>),
    /// A code block.
    CodeBlock(MarkdownCodeBlockKind),

    /// A list. If the list is ordered the field indicates the number of the first item.
    /// Contains only list items.
    List(Option<u64>), // TODO: add delim and tight for ast (not needed for html)
    /// A list item.
    Item,
    /// A footnote definition. The value contained is the footnote's label by which it can
    /// be referred to.
    FootnoteDefinition(String),

    /// A table. Contains a vector describing the text-alignment for each of its columns.
    Table(Vec<MarkdownAlignment>),
    /// A table header. Contains only `TableRow`s. Note that the table body starts immediately
    /// after the closure of the `TableHead` tag. There is no `TableBody` tag.
    TableHead,
    /// A table row. Is used both for header rows as body rows. Contains only `TableCell`s.
    TableRow,
    TableCell,

    // span-level tags
    Emphasis,
    Strong,
    Strikethrough,

    /// A link. The first field is the link type, the second the destination URL, the third is a title and the fourth is the identifier.
    Link(MarkdownLinkType, String, String, String),
    /// An image. The first field is the link type, the second the destination URL, the third is a title and fourth is the identifier.
    Image(MarkdownLinkType, String, String, String),
    HtmlBlock,
    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,
    MetadataBlock(MarkdownMetadataBlockKind),
}

impl<'a> From<Tag<'a>> for MarkdownTag {
    fn from(x: Tag<'a>) -> Self {
        match x {
            Tag::Paragraph => Self::Paragraph,
            Tag::Heading {
                level: x,
                id: y,
                classes: z,
                attrs: a,
            } => Self::Heading(
                x.into(),
                y.map(Into::into),
                z.into_iter().map(Into::into).collect(),
                a.into_iter().map(Into::into).collect(),
            ),
            Tag::BlockQuote(x) => Self::BlockQuote(x.map(Into::into)),
            Tag::CodeBlock(x) => Self::CodeBlock(x.into()),
            Tag::List(x) => Self::List(x),
            Tag::Item => Self::Item,
            Tag::FootnoteDefinition(x) => Self::FootnoteDefinition(x.to_string()),
            Tag::Table(x) => Self::Table(x.into_iter().map(Into::into).collect()),
            Tag::TableHead => Self::TableHead,
            Tag::TableRow => Self::TableRow,
            Tag::TableCell => Self::TableCell,
            Tag::Emphasis => Self::Emphasis,
            Tag::Strong => Self::Strong,
            Tag::Strikethrough => Self::Strikethrough,
            Tag::Link {
                link_type: x,
                dest_url: y,
                title: z,
                id: a,
            } => Self::Link(x.into(), y.to_string(), z.to_string(), a.to_string()),
            Tag::Image {
                link_type: x,
                dest_url: y,
                title: z,
                id: a,
            } => Self::Image(x.into(), y.to_string(), z.to_string(), a.into_string()),
            Tag::HtmlBlock => Self::HtmlBlock,
            Tag::DefinitionList => Self::DefinitionList,
            Tag::DefinitionListTitle => Self::DefinitionListTitle,
            Tag::DefinitionListDefinition => Self::DefinitionListDefinition,
            Tag::MetadataBlock(metadata_block_kind) => {
                Self::MetadataBlock(metadata_block_kind.into())
            }
        }
    }
}

impl From<MarkdownTag> for Tag<'static> {
    fn from(x: MarkdownTag) -> Self {
        match x {
            MarkdownTag::Paragraph => Self::Paragraph,
            MarkdownTag::Heading(x, y, z, a) => Self::Heading {
                level: x.into(),
                id: y.map(Into::into),
                classes: z.into_iter().map(Into::into).collect(),
                attrs: a.into_iter().map(Into::into).collect(),
            },
            MarkdownTag::BlockQuote(x) => Self::BlockQuote(x.map(Into::into)),
            MarkdownTag::CodeBlock(x) => Self::CodeBlock(x.into()),
            MarkdownTag::List(x) => Self::List(x),
            MarkdownTag::Item => Self::Item,
            MarkdownTag::FootnoteDefinition(x) => Self::FootnoteDefinition(x.into()),
            MarkdownTag::Table(x) => Self::Table(x.into_iter().map(Into::into).collect()),
            MarkdownTag::TableHead => Self::TableHead,
            MarkdownTag::TableRow => Self::TableRow,
            MarkdownTag::TableCell => Self::TableCell,
            MarkdownTag::Emphasis => Self::Emphasis,
            MarkdownTag::Strong => Self::Strong,
            MarkdownTag::Strikethrough => Self::Strikethrough,
            MarkdownTag::Link(x, y, z, a) => Self::Link {
                link_type: x.into(),
                dest_url: y.into(),
                title: z.into(),
                id: a.into(),
            },
            MarkdownTag::Image(x, y, z, id) => Self::Image {
                link_type: x.into(),
                dest_url: y.into(),
                title: z.into(),
                id: id.into(),
            },
            MarkdownTag::HtmlBlock => Self::HtmlBlock,
            MarkdownTag::DefinitionList => Self::DefinitionList,
            MarkdownTag::DefinitionListTitle => Self::DefinitionListTitle,
            MarkdownTag::DefinitionListDefinition => Self::DefinitionListDefinition,
            MarkdownTag::MetadataBlock(markdown_metadata_block_kind) => {
                Self::MetadataBlock(markdown_metadata_block_kind.into())
            }
        }
    }
}

#[derive(Clone, FromToLua, Debug, ToTypename)]
/// Markdown events that are generated in a pre-order traversal of the document
/// tree, with additional `End` events whenever all of an inner node's children
/// have been visited.
pub enum MarkdownEvent {
    Start(MarkdownTag),
    End(MarkdownTagEnd),
    Text(String),
    Code(String),
    Html(String),
    FootnoteReference(String),
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker(bool),
    InlineMath(String),
    DisplayMath(String),
    InlineHtml(String),
}

impl tealr::mlu::FromLuaExact for MarkdownEvent {
    fn from_lua_exact(
        value: tealr::mlu::mlua::Value,
        lua: &tealr::mlu::mlua::Lua,
    ) -> tealr::mlu::mlua::Result<Self> {
        MarkdownEvent::from_lua(value, lua)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, ToTypename, FromToLua)]
pub enum MarkdownTagEnd {
    Paragraph,
    Heading(MarkdownHeadingLevel),

    BlockQuote(Option<MarkdownBlockQuoteKind>),
    CodeBlock,

    HtmlBlock,

    /// A list, `true` for ordered lists.
    List(bool),
    Item,
    FootnoteDefinition,

    DefinitionList,
    DefinitionListTitle,
    DefinitionListDefinition,

    Table,
    TableHead,
    TableRow,
    TableCell,

    Emphasis,
    Strong,
    Strikethrough,

    Link,
    Image,

    MetadataBlock(MarkdownMetadataBlockKind),
}

impl From<MarkdownTagEnd> for TagEnd {
    fn from(x: MarkdownTagEnd) -> Self {
        match x {
            MarkdownTagEnd::Paragraph => Self::Paragraph,
            MarkdownTagEnd::Heading(x) => Self::Heading(x.into()),
            MarkdownTagEnd::BlockQuote(x) => Self::BlockQuote(x.map(Into::into)),
            MarkdownTagEnd::CodeBlock => Self::CodeBlock,
            MarkdownTagEnd::HtmlBlock => Self::HtmlBlock,
            MarkdownTagEnd::List(x) => Self::List(x),
            MarkdownTagEnd::Item => Self::Item,
            MarkdownTagEnd::FootnoteDefinition => Self::FootnoteDefinition,
            MarkdownTagEnd::DefinitionList => Self::DefinitionList,
            MarkdownTagEnd::DefinitionListTitle => Self::DefinitionListTitle,
            MarkdownTagEnd::DefinitionListDefinition => Self::DefinitionListDefinition,
            MarkdownTagEnd::Table => Self::Table,
            MarkdownTagEnd::TableHead => Self::TableHead,
            MarkdownTagEnd::TableRow => Self::TableRow,
            MarkdownTagEnd::TableCell => Self::TableCell,
            MarkdownTagEnd::Emphasis => Self::Emphasis,
            MarkdownTagEnd::Strong => Self::Strong,
            MarkdownTagEnd::Strikethrough => Self::Strikethrough,
            MarkdownTagEnd::Link => Self::Link,
            MarkdownTagEnd::Image => Self::Image,
            MarkdownTagEnd::MetadataBlock(x) => Self::MetadataBlock(x.into()),
        }
    }
}

impl From<TagEnd> for MarkdownTagEnd {
    fn from(x: TagEnd) -> Self {
        match x {
            TagEnd::Paragraph => Self::Paragraph,
            TagEnd::Heading(x) => Self::Heading(x.into()),
            TagEnd::BlockQuote(x) => Self::BlockQuote(x.map(Into::into)),
            TagEnd::CodeBlock => Self::CodeBlock,
            TagEnd::HtmlBlock => Self::HtmlBlock,
            TagEnd::List(x) => Self::List(x),
            TagEnd::Item => Self::Item,
            TagEnd::FootnoteDefinition => Self::FootnoteDefinition,
            TagEnd::DefinitionList => Self::DefinitionList,
            TagEnd::DefinitionListTitle => Self::DefinitionListTitle,
            TagEnd::DefinitionListDefinition => Self::DefinitionListDefinition,
            TagEnd::Table => Self::Table,
            TagEnd::TableHead => Self::TableHead,
            TagEnd::TableRow => Self::TableRow,
            TagEnd::TableCell => Self::TableCell,
            TagEnd::Emphasis => Self::Emphasis,
            TagEnd::Strong => Self::Strong,
            TagEnd::Strikethrough => Self::Strikethrough,
            TagEnd::Link => Self::Link,
            TagEnd::Image => Self::Image,
            TagEnd::MetadataBlock(x) => Self::MetadataBlock(x.into()),
        }
    }
}

impl From<Event<'_>> for MarkdownEvent {
    fn from(x: Event) -> Self {
        match x {
            Event::Start(x) => Self::Start(x.into()),
            Event::End(x) => Self::End(x.into()),
            Event::Text(x) => Self::Text(x.to_string()),
            Event::Code(x) => Self::Code(x.to_string()),
            Event::Html(x) => Self::Html(x.to_string()),
            Event::FootnoteReference(x) => Self::FootnoteReference(x.to_string()),
            Event::SoftBreak => Self::SoftBreak,
            Event::HardBreak => Self::HardBreak,
            Event::Rule => Self::Rule,
            Event::TaskListMarker(x) => Self::TaskListMarker(x),
            Event::InlineMath(cow_str) => Self::InlineMath(cow_str.to_string()),
            Event::DisplayMath(cow_str) => Self::DisplayMath(cow_str.to_string()),
            Event::InlineHtml(cow_str) => Self::InlineHtml(cow_str.to_string()),
        }
    }
}
impl From<MarkdownEvent> for Event<'static> {
    fn from(x: MarkdownEvent) -> Self {
        match x {
            MarkdownEvent::Start(x) => Self::Start(x.into()),
            MarkdownEvent::End(x) => Self::End(x.into()),
            MarkdownEvent::Text(x) => Self::Text(CowStr::from(x)),
            MarkdownEvent::Code(x) => Self::Code(CowStr::from(x)),
            MarkdownEvent::Html(x) => Self::Html(CowStr::from(x)),
            MarkdownEvent::FootnoteReference(x) => Self::FootnoteReference(x.into()),
            MarkdownEvent::SoftBreak => Self::SoftBreak,
            MarkdownEvent::HardBreak => Self::HardBreak,
            MarkdownEvent::Rule => Self::Rule,
            MarkdownEvent::TaskListMarker(x) => Self::TaskListMarker(x),
            MarkdownEvent::InlineMath(x) => Self::InlineMath(x.into()),
            MarkdownEvent::DisplayMath(x) => Self::DisplayMath(x.into()),
            MarkdownEvent::InlineHtml(x) => Self::InlineHtml(x.into()),
        }
    }
}

/// This parses markdown, runs the given function for each event and then returns the full html.
///
/// It is made to be used from lua.
///
/// It basically does `parser::new_ext().flat_map(func).collect()`,
/// though with a bit more internal ceremony to get it all working
pub fn parse_markdown_lua(
    to_parse: String,
    func: impl Fn(MarkdownEvent) -> Result<Vec<MarkdownEvent>, tealr::mlu::mlua::Error>,
) -> Result<String, tealr::mlu::mlua::Error> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&to_parse, options);
    let injected = parser
        .flat_map(|x| {
            let z = func(x.into());
            match z {
                Ok(x) => x.into_iter().map(Ok).collect::<Vec<_>>(),
                Err(x) => vec![Err(x)],
            }
        })
        .collect::<Result<Vec<_>, tealr::mlu::mlua::Error>>()?;
    let transformed = injected.iter().map(|v| match v {
        MarkdownEvent::Start(MarkdownTag::Heading(x, y, z, a)) => Event::Start(Tag::Heading {
            level: (*x).into(),
            id: y.as_deref().map(Into::into),
            classes: z.iter().map(|x| x.to_string().into()).collect(),
            attrs: a.iter().cloned().map(Into::into).collect(),
        }),
        MarkdownEvent::End(MarkdownTagEnd::Heading(x)) => Event::End(TagEnd::Heading((*x).into())),
        x => x.to_owned().into(),
    });
    let mut html_output = Default::default();
    html::push_html(&mut html_output, transformed);
    Ok(html_output)
}

type MarkdownEventTable = Vec<MarkdownEvent>;
type OptionalMarkdownEvent = Option<MarkdownEvent>;

tealr::create_union_mlua!(pub enum MarkdownTransformation = MarkdownEventTable | OptionalMarkdownEvent );
