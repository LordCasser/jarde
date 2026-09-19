//! The naming decisions of the Java presentation (P3 3.1's minimal stable subset, landed here so
//! that 1.3's closed loop already writes legal text).
//!
//! # What the decision is and is not
//!
//! The layer below owns the *evidence*: a debug name out of `LocalVariableTable`/`MethodParameters`
//! when the class file has one, a resolution result, a descriptor. This module owns the *spelling*:
//! which text stands in for one local slot, and whether that text is the original name or an alias.
//!
//! Three rules, all of them deterministic — the same request produces the same names, which is what
//! makes a recovered artifact comparable across runs:
//!
//! 1. **No evidence is not a failure.** A body compiled with `-g:none` has no name for a slot at
//!    all. The slot still gets a name, derived from its ordinal (`arg0…`, `local0…`), because the
//!    alternative — an unnamed local, or a hard error — would drop a body Java can express.
//!    An invented name is *not* an alias: nothing was hidden, so it does not by itself move the
//!    syntax plane.
//! 2. **A name Java cannot spell becomes an alias, never an error.** Keywords, `true`/`false`/
//!    `null`, `const`/`goto`, an empty name, obfuscated names with characters outside the
//!    identifier grammar, and names starting with a digit all take [`alias_for`], which is a pure
//!    function of the raw spelling. The original stays as evidence; the text carries the alias. This
//!    is the case the P3 vocabulary records as `representation = Java` with `syntax_status =
//!    NotJava`: the presentation is Java, but it is not claimed to be a faithful spelling of a legal
//!    Java program.
//! 3. **Two slots never share one name.** A collision is resolved by slot order with a numeric
//!    suffix, so the mapping is a function of the table and not of hash order.
//!
//! The grammar accepted here is the practical subset for names that reach a recovered body: ASCII
//! `$`/`_`/letters followed by those plus digits. A non-ASCII identifier character is treated as
//! unspellable and aliased — widening the accepted grammar is a 3.1 question with its own corpus
//! (multi-generation javac/ECJ output), and guessing it here would mean emitting identifiers this
//! layer cannot check.

use std::collections::{BTreeMap, BTreeSet};

/// Words Java does not accept as an identifier, including the literals and the two reserved words.
///
/// `_` is in the list because a single underscore is a keyword from Java 9 on; a recovered artifact
/// is Java 8 (the release this project targets), where `_` is legal but warned about — alias it
/// anyway, because a name that is legal in one release and illegal in the next is exactly the kind
/// of input this list exists to make deterministic.
pub const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "void",
    "volatile",
    "while",
    "_",
];

/// Whether `text` is an identifier this layer is willing to write.
pub fn is_java_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    let start = first == '_' || first == '$' || first.is_ascii_alphabetic();
    start
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
        && !JAVA_KEYWORDS.contains(&text)
}

/// The deterministic alias of a raw name Java cannot spell.
///
/// The result is a function of `raw` alone, so two runs that read the same evidence write the same
/// text, and it is always a legal identifier that is not a keyword: an appended `_` cannot create a
/// keyword, because no keyword ends in one.
pub fn alias_for(raw: &str) -> String {
    let mut alias = String::with_capacity(raw.len() + 1);
    for (index, character) in raw.chars().enumerate() {
        let legal = if index == 0 {
            character == '_' || character == '$' || character.is_ascii_alphabetic()
        } else {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        };
        alias.push(if legal { character } else { '_' });
    }
    if alias.is_empty() {
        alias.push_str("unnamed");
    }
    if JAVA_KEYWORDS.contains(&alias.as_str()) {
        alias.push('_');
    }
    debug_assert!(
        is_java_identifier(&alias),
        "an alias is always spellable: {alias}"
    );
    alias
}

/// The name one local slot is written as, with the evidence it came from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedName {
    text: String,
    raw: Option<String>,
    aliased: Option<AliasReason>,
    slot: u16,
}

/// Why a raw name was not written as it stood.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AliasReason {
    /// The raw spelling is a keyword, a literal, or outside the identifier grammar.
    Unspellable,
    /// Another slot already took the spelling this one would have had.
    Collision,
}

impl RenderedName {
    /// The text the presentation writes.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The evidence this name came from, when the body had any.
    pub fn raw(&self) -> Option<&str> {
        self.raw.as_deref()
    }

    /// Why the text is not the raw spelling, when it is not.
    pub fn aliased(&self) -> Option<AliasReason> {
        self.aliased
    }

    /// The slot this name stands for.
    pub fn slot(&self) -> u16 {
        self.slot
    }
}

/// The names of one method's local slots, decided in slot order.
///
/// Built from the declared parameter count (parameter slots are the first ones a body's locals
/// array holds) and the raw debug names the body carries, if any. Slots above the ones the table
/// states have no name at all: a node that writes one is unrenderable evidence, and the callers of
/// this table treat it as such instead of inventing a spelling for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameTable {
    names: BTreeMap<u16, RenderedName>,
    aliased: usize,
    invented: usize,
}

impl NameTable {
    /// One row per slot, decided in slot order so that collisions resolve deterministically.
    ///
    /// `parameters` is the number of parameter slots (the method's own arity in slots, `this`
    /// included for an instance method when the caller counted it that way), `slots` is how many
    /// local slots the body has, and `debug` holds the raw name of slot `i` at index `i` when the
    /// body states one. The table states a name for every slot the body has: it is the local slots
    /// the body's frames declare, not the debug names it happens to carry, that say which slots a
    /// recovered statement can mention.
    pub fn build(parameters: u16, slots: u16, debug: &[Option<String>]) -> Self {
        let mut table = Self {
            names: BTreeMap::new(),
            aliased: 0,
            invented: 0,
        };
        let slots = slots
            .max(parameters)
            .max(u16::try_from(debug.len()).unwrap_or(u16::MAX));
        let mut taken: BTreeMap<String, u16> = BTreeMap::new();
        for slot in 0..slots {
            let raw = debug
                .get(usize::from(slot))
                .and_then(|name| name.as_deref())
                .map(str::to_owned);
            let (text, aliased) = match &raw {
                None => {
                    table.invented += 1;
                    (invented_name(slot, parameters), None)
                }
                Some(raw) if is_java_identifier(raw) => (raw.clone(), None),
                Some(raw) => (alias_for(raw), Some(AliasReason::Unspellable)),
            };
            let (text, aliased) = match taken.get(&text) {
                None => (text, aliased),
                Some(_) => {
                    let mut suffix = 2u32;
                    loop {
                        let candidate = format!("{text}_{suffix}");
                        if !taken.contains_key(&candidate) {
                            break (candidate, Some(AliasReason::Collision));
                        }
                        suffix += 1;
                    }
                }
            };
            if aliased.is_some() {
                table.aliased += 1;
            }
            taken.insert(text.clone(), slot);
            table.names.insert(
                slot,
                RenderedName {
                    text,
                    raw,
                    aliased,
                    slot,
                },
            );
        }
        table
    }

    /// The name of one slot, when the table states one.
    pub fn name(&self, slot: u16) -> Option<&RenderedName> {
        self.names.get(&slot)
    }

    /// The first spelling of the form `base`, `base_`, `base__`, … that no local of this body
    /// already carries and that Java accepts as an identifier.
    ///
    /// A name a *shape* invents for something that is not one of the body's slots — a lambda's
    /// parameters (P3 2.1) — has to differ from every name the locals were given, because JLS 6.4
    /// forbids a lambda parameter that shadows an enclosing local. The search is deterministic and
    /// independent of the order the slots were named in: one candidate order, first free one wins.
    /// The base itself is checked like every other candidate, so a caller whose base is a keyword or
    /// an unspellable spelling gets the same treatment a debug name would.
    pub fn free_name(&self, base: &str) -> String {
        let taken: BTreeSet<&str> = self.names.values().map(|name| name.text.as_str()).collect();
        let mut candidate = base.to_string();
        while taken.contains(candidate.as_str()) || !is_java_identifier(&candidate) {
            candidate.push('_');
        }
        candidate
    }

    /// The text of one slot's name, when the table states one.
    pub fn text(&self, slot: u16) -> Option<&str> {
        self.names.get(&slot).map(RenderedName::text)
    }

    /// How many slots could not be written as their evidence spelled them.
    pub fn aliased(&self) -> usize {
        self.aliased
    }

    /// How many slots had no evidence at all and got an ordinal name.
    pub fn invented(&self) -> usize {
        self.invented
    }

    /// Whether any slot's text differs from the spelling its evidence carried.
    ///
    /// This is the one bit the syntax plane reads: an alias means the produced text is not claimed
    /// to spell the input faithfully, whether or not it is legal Java.
    pub fn any_aliased(&self) -> bool {
        self.aliased > 0
    }

    /// Every name the table decided, in slot order.
    pub fn names(&self) -> impl Iterator<Item = &RenderedName> {
        self.names.values()
    }
}

/// The name of a slot no evidence names: parameters first, then the locals above them.
fn invented_name(slot: u16, parameters: u16) -> String {
    if slot < parameters {
        format!("arg{slot}")
    } else {
        format!("local{slot}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyword_becomes_a_deterministic_alias_and_never_an_error() {
        assert_eq!(alias_for("int"), "int_");
        assert_eq!(alias_for("int"), alias_for("int"), "pure in the spelling");
        assert_eq!(alias_for(""), "unnamed");
        assert_eq!(alias_for("1x"), "_x");
        assert_eq!(alias_for("a.b"), "a_b");
        assert_eq!(
            alias_for("😀"),
            "__",
            "a `_` is itself a keyword, so the alias gets a second"
        );
        assert!(is_java_identifier(&alias_for("while")));
        assert!(!is_java_identifier("while"));
        assert!(!is_java_identifier("true"));
        assert!(is_java_identifier("$value_2"));
    }

    #[test]
    fn no_debug_evidence_still_names_every_slot_deterministically() {
        let table = NameTable::build(2, 4, &[]);
        assert_eq!(table.text(0), Some("arg0"));
        assert_eq!(table.text(1), Some("arg1"));
        assert_eq!(table.text(2), Some("local2"));
        assert_eq!(table.text(3), Some("local3"));
        assert_eq!(
            table.text(4),
            None,
            "a slot the body does not have has no name"
        );
        assert_eq!(table.invented(), 4);
        assert!(!table.any_aliased(), "an invented name hides nothing");
        assert_eq!(
            NameTable::build(2, 4, &[]),
            table,
            "the same evidence, the same table"
        );
    }

    #[test]
    fn evidence_wins_where_it_exists_and_aliases_where_it_cannot_be_written() {
        let debug = vec![Some("int".to_string()), None, Some("count".to_string())];
        let table = NameTable::build(1, 3, &debug);
        assert_eq!(table.name(0).and_then(RenderedName::raw), Some("int"));
        assert_eq!(table.text(0), Some("int_"));
        assert_eq!(
            table.name(0).and_then(RenderedName::aliased),
            Some(AliasReason::Unspellable)
        );
        assert_eq!(table.text(1), Some("local1"), "the slot without evidence");
        assert_eq!(table.text(2), Some("count"));
        assert_eq!(table.aliased(), 1);
        assert!(table.any_aliased());
    }

    #[test]
    fn a_collision_resolves_by_slot_order_and_is_an_alias() {
        let debug = vec![Some("value".to_string()), Some("value".to_string())];
        let table = NameTable::build(0, 2, &debug);
        assert_eq!(table.text(0), Some("value"));
        assert_eq!(table.text(1), Some("value_2"));
        assert_eq!(
            table.name(1).and_then(RenderedName::aliased),
            Some(AliasReason::Collision)
        );
        assert_eq!(table.aliased(), 1);
    }
}
