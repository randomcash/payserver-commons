//! The page descriptor: a typed component tree a plugin returns instead of
//! markup, drawn by the host's own client.
//!
//! A plugin cannot ship Rust into an already-compiled Leptos client, so it
//! ships data. The vocabulary mirrors ui-kit's own components - card, badge,
//! button, table, form, input, select, notice, tabs, plus the layout
//! primitives stack, row, grid and section - so there is nothing the
//! merchant dashboard renders that a plugin page cannot.
//!
//! Static rendering, with one way out. There is no action system: a plugin
//! cannot make a button POST back into itself, and adding one means CSRF,
//! idempotency and an action-routing design that the money path deserves
//! more care over than a first slice can give it.
//!
//! What a button can do is *link*. That is enough for the case that
//! actually blocks revenue - a plugin issues an invoice through the host's
//! `invoice_create` import and points the merchant at the checkout the host
//! already serves - without inventing a second way to move money. A button
//! with no `href` still renders and still does nothing, and the client is
//! expected to draw it disabled rather than leaving a control that looks
//! live and is not.

use serde::{Deserialize, Serialize};

/// Who the host says is asking.
///
/// The plugin decides what to return for a given [`Viewer`]; it never
/// chooses which one it is. The same endpoint serves merchant and admin
/// views - the host resolves this from the authenticated identity, not from
/// anything the request itself claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Viewer {
    Merchant,
    Admin,
}

/// A node in a plugin page.
///
/// Variant names are a public contract, not a rename target: when ui-kit
/// renames one of its own components internally, this enum keeps the old
/// name and the client's renderer moves the mapping in a `match`. A plugin
/// ships against this vocabulary, not against ui-kit's Rust API.
///
/// `Unknown` is what makes the vocabulary safe to grow. `#[serde(other)]`
/// requires a unit variant, so it cannot carry the `type` string it failed to
/// match - only the fact that nothing matched. That is enough: the client's
/// renderer draws a visible placeholder for it, never nothing. A silently
/// blank panel beside a paywall is the worst failure this crate can produce,
/// and it is exactly what a newer plugin against an older client would give
/// you without this variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PageElement {
    Card(Card),
    /// Plain prose. The thing this vocabulary was missing.
    ///
    /// Without it every piece of text had to be a `Badge`, so field labels
    /// and values rendered as coloured pills - which looked nothing like the
    /// host's own screens and made a plugin page identifiable at a glance as
    /// the odd one out.
    Text(Text),
    /// Label-and-value pairs, the shape most of a detail page actually is.
    ///
    /// A distinct element rather than a two-column `Table` because a client
    /// should be free to draw it as a definition list, a two-column grid or
    /// stacked rows depending on width - and because a table of two cells per
    /// row, with headers a reader does not need, is a heavier thing than what
    /// is being said.
    Fields(Fields),
    Badge(Badge),
    Button(Button),
    Table(Table),
    Form(Form),
    Input(Input),
    Select(Select),
    Notice(Notice),
    Tabs(Tabs),
    Stack(Stack),
    Row(Row),
    Grid(Grid),
    Section(Section),
    #[serde(other)]
    Unknown,
}

/// A visual tone shared by the elements that carry one (badge, notice).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

/// Mirrors ui-kit's `ButtonVariant`. Kept as a separate type rather than a
/// shared one, per the same contract-stability rule as [`PageElement`]
/// itself: ui-kit's enum is free to change shape without a plugin rebuild.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
}

/// Layout direction for a [`Stack`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Column,
    Row,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Card {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A status to show beside the title, in the card's own header.
    ///
    /// Separate from [`children`](Self::children) because it is not content:
    /// it is what this card is *about* right now, and every screen in the
    /// host puts that on the header line opposite the title rather than as
    /// the first thing in the body. A plugin with no way to say it had to
    /// push a `Badge` into the body, where it reads as a stray pill above the
    /// detail instead of a status on the thing.
    ///
    /// Optional and defaulted, so a card written before this existed
    /// deserialises unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<Badge>,
    #[serde(default)]
    pub children: Vec<PageElement>,
}

/// How prominent a piece of text is.
///
/// Not a font size: the host picks the sizes. This says what the text is
/// doing, so a client can stay consistent with its own screens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    /// Ordinary prose.
    #[default]
    Body,
    /// Quieter than body - a hint, a caveat, a unit.
    Muted,
    /// A figure worth reading first: an amount, a count.
    Strong,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub text: String,
    #[serde(default)]
    pub style: TextStyle,
}

/// One label and its value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fields {
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    pub text: String,
    #[serde(default)]
    pub tone: Tone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub label: String,
    #[serde(default)]
    pub variant: ButtonVariant,
    /// Where the button goes, if anywhere.
    ///
    /// A path on this host, not a URL: the server does not reliably know its
    /// own external origin, and a plugin that rendered a wrong absolute URL
    /// would send a paying merchant somewhere that is not this instance. A
    /// client must treat a value that is not a host-relative path as no link
    /// at all - a plugin is not a trusted source of somewhere to send a
    /// merchant with money in hand.
    ///
    /// `None` is a button that does nothing, and a client must draw it
    /// disabled. A control that looks live and is not is worse than one that
    /// is visibly unavailable, and this vocabulary has no other way to say
    /// "not yet".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Table {
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Form {
    #[serde(default)]
    pub children: Vec<PageElement>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Input {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Select {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notice {
    pub text: String,
    #[serde(default)]
    pub tone: Tone,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub label: String,
    #[serde(default)]
    pub content: Vec<PageElement>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Tabs {
    #[serde(default)]
    pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub children: Vec<PageElement>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Row {
    #[serde(default)]
    pub children: Vec<PageElement>,
}

/// Grid defaults to two columns - one is indistinguishable from a stack, and
/// a plugin that wants three or more says so explicitly.
fn default_columns() -> u8 {
    2
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    #[serde(default = "default_columns")]
    pub columns: u8,
    #[serde(default)]
    pub children: Vec<PageElement>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            columns: default_columns(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Section {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default)]
    pub children: Vec<PageElement>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(element: &PageElement) {
        let json = serde_json::to_string(element).unwrap();
        let back: PageElement = serde_json::from_str(&json).unwrap();
        assert_eq!(element, &back, "did not round-trip through {json}");
    }

    /// A card serialised before `badge` existed still reads.
    ///
    /// The vocabulary grows and a host may hold descriptors a plugin sent
    /// before the field was added - so an absent `badge` has to mean "no
    /// badge", not "this card is unreadable". The field is also skipped on
    /// the way out when it is `None`, so adding it changed nothing about what
    /// an existing card serialises to.
    #[test]
    fn a_card_without_a_badge_still_deserialises_and_does_not_grow_one() {
        let old = r#"{"type":"card","title":"Balances","children":[]}"#;
        let card: PageElement = serde_json::from_str(old).expect("an older card must still read");
        assert_eq!(
            card,
            PageElement::Card(Card {
                title: Some("Balances".to_string()),
                badge: None,
                children: Vec::new(),
            })
        );
        assert_eq!(
            serde_json::to_string(&card).unwrap(),
            old,
            "a card with no badge must serialise exactly as it did before the field existed"
        );
    }

    /// Every component in the vocabulary round-trips: descriptor -> JSON ->
    /// descriptor. This is the data-level half of the ticket's round-trip
    /// requirement; the client's renderer test covers JSON -> markup.
    #[test]
    fn every_vocabulary_element_round_trips() {
        round_trip(&PageElement::Card(Card {
            title: Some("Balances".to_string()),
            badge: Some(Badge {
                text: "Live".to_string(),
                tone: Tone::Success,
            }),
            children: vec![PageElement::Text(Text {
                text: "Updated a moment ago".to_string(),
                style: TextStyle::Muted,
            })],
        }));
        round_trip(&PageElement::Badge(Badge {
            text: "Beta".to_string(),
            tone: Tone::Info,
        }));
        round_trip(&PageElement::Button(Button {
            label: "Refresh".to_string(),
            variant: ButtonVariant::Outline,
            href: None,
        }));
        round_trip(&PageElement::Button(Button {
            label: "Pay now".to_string(),
            variant: ButtonVariant::Primary,
            href: Some("/checkout/9f3a".to_string()),
        }));
        round_trip(&PageElement::Table(Table {
            headers: vec!["Chain".to_string(), "Balance".to_string()],
            rows: vec![vec!["eip155:1".to_string(), "1.2".to_string()]],
        }));
        round_trip(&PageElement::Form(Form {
            children: vec![PageElement::Input(Input {
                label: Some("Amount".to_string()),
                placeholder: "0.00".to_string(),
            })],
        }));
        round_trip(&PageElement::Input(Input {
            label: None,
            placeholder: "Search".to_string(),
        }));
        round_trip(&PageElement::Select(Select {
            label: Some("Network".to_string()),
            options: vec!["Ethereum".to_string(), "Polygon".to_string()],
        }));
        round_trip(&PageElement::Notice(Notice {
            text: "Read-only preview".to_string(),
            tone: Tone::Warning,
        }));
        round_trip(&PageElement::Tabs(Tabs {
            tabs: vec![Tab {
                label: "Overview".to_string(),
                content: vec![],
            }],
        }));
        round_trip(&PageElement::Stack(Stack {
            direction: Direction::Row,
            children: vec![],
        }));
        round_trip(&PageElement::Row(Row { children: vec![] }));
        round_trip(&PageElement::Grid(Grid {
            columns: 3,
            children: vec![],
        }));
        round_trip(&PageElement::Section(Section {
            title: Some("Advanced".to_string()),
            children: vec![],
        }));
    }

    /// A `type` this build does not recognise must not fail the whole page.
    /// This is the property that lets an older client parse a page from a
    /// newer plugin: the unrecognised node becomes `Unknown` instead of an
    /// error that would take the rest of the tree down with it.
    #[test]
    fn unrecognised_type_deserializes_as_unknown_instead_of_failing() {
        let json = r#"{"type": "chart", "series": [1, 2, 3]}"#;
        let element: PageElement = serde_json::from_str(json).unwrap();
        assert_eq!(element, PageElement::Unknown);
    }

    /// `Unknown` sits beside a known sibling in a tree without disturbing it -
    /// the whole point of not failing the parse is that the rest of the page
    /// still renders.
    #[test]
    fn unknown_element_nests_inside_a_known_container() {
        let json = r#"{
            "type": "stack",
            "children": [
                {"type": "badge", "text": "ok", "tone": "success"},
                {"type": "chart", "series": [1, 2, 3]}
            ]
        }"#;
        let element: PageElement = serde_json::from_str(json).unwrap();
        let PageElement::Stack(stack) = element else {
            panic!("expected a stack");
        };
        assert_eq!(stack.children.len(), 2);
        assert_eq!(stack.children[1], PageElement::Unknown);
    }

    #[test]
    fn viewer_round_trips_and_is_not_client_chosen() {
        assert_eq!(
            serde_json::from_str::<Viewer>(r#""admin""#).unwrap(),
            Viewer::Admin
        );
        assert_eq!(
            serde_json::from_str::<Viewer>(r#""merchant""#).unwrap(),
            Viewer::Merchant
        );
    }

    /// A missing optional field (`tone`, `variant`, `direction`, ...) must
    /// deserialize via its default rather than being required - a minimal
    /// plugin payload should not have to spell out every default.
    #[test]
    fn optional_fields_default_when_omitted() {
        let json = r#"{"type": "button", "label": "Go"}"#;
        let element: PageElement = serde_json::from_str(json).unwrap();
        assert_eq!(
            element,
            PageElement::Button(Button {
                label: "Go".to_string(),
                variant: ButtonVariant::Primary,
                href: None,
            })
        );
    }

    /// The gap these two close. Before `Text` and `Fields`, a plugin with a
    /// label and a value had only `Badge` to say it with, so detail pages
    /// rendered as rows of coloured pills - instantly identifiable as not
    /// part of the host's own interface.
    #[test]
    fn prose_and_label_value_pairs_do_not_have_to_be_badges() {
        round_trip(&PageElement::Text(Text {
            text: "Your subscription starts once the first invoice is paid.".to_string(),
            style: TextStyle::Body,
        }));
        round_trip(&PageElement::Fields(Fields {
            fields: vec![
                Field {
                    label: "Price".to_string(),
                    value: "0.50 USDC every 30 days".to_string(),
                },
                Field {
                    label: "Renews".to_string(),
                    value: "18 October 2026".to_string(),
                },
            ],
        }));
    }

    /// Both default, so a plugin that says only what it means gets sensible
    /// output, and an older plugin's JSON keeps parsing.
    #[test]
    fn text_defaults_to_body_and_fields_to_empty() {
        let text: Text = serde_json::from_str(r#"{"text":"hello"}"#).unwrap();
        assert_eq!(text.style, TextStyle::Body);

        let fields: Fields = serde_json::from_str("{}").unwrap();
        assert!(fields.fields.is_empty());
    }
}
