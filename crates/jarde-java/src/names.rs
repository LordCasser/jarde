//! The naming decisions of the Java presentation (P3 3.1's minimal stable subset, landed here so
//! that 1.3's closed loop already writes legal text).
//!
//! # What the decision is and is not
//!
//! The layer below owns the *evidence*: a debug name out of `LocalVariableTable`/`MethodParameters`
//! when the class file has one, a resolution result, a descriptor. This module owns the *spelling*:
//! which text stands in for one local slot, and whether that text is the original name or an alias.
//!
//! Four rules, all of them deterministic — the same request produces the same names, which is what
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
//! 3. **Two variables never share one name.** A collision is resolved by slot order — and, inside
//!    one slot, by variable order — with a numeric suffix, so the mapping is a function of the
//!    table and not of hash order.
//! 4. **One slot can be several variables.** A compiler reuses one storage location for two
//!    variables whose scopes do not overlap, and the table's records say so: two ranges, two names.
//!    The produced text then has two variables with their own names, each declared where its own
//!    uses can see it — *not* one variable for the whole slot. Which uses belong to which variable
//!    is [`crate::reuse`]'s decision, taken from the same run's own SSA; this module only decides the
//!    spelling of the variables that decision states.
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

/// One name a `LocalVariableTable` states for one local slot over one range of bytecode (P3 3.1).
///
/// The record is **evidence**, never a decision: it states that slot `slot` carried a variable the
/// source called `name` over the bytecode range `[start_bci, end_bci)`. Two records for one slot are
/// exactly what a compiler that reuses a storage location for two variables in disjoint scopes
/// writes, and whether that becomes two variables in the produced text is [`crate::reuse`]'s
/// decision — taken from the slot's own uses — rather than this record's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugLocal {
    slot: u16,
    name: String,
    range: Option<(u32, u32)>,
}

impl DebugLocal {
    /// A record naming one slot over the bytecode range `[start_bci, end_bci)`.
    pub fn over(slot: u16, name: impl Into<String>, start_bci: u32, end_bci: u32) -> Self {
        Self {
            slot,
            name: name.into(),
            range: Some((start_bci, end_bci)),
        }
    }

    /// A name for one slot with no range stated: what an entry point that was handed one name per
    /// slot states. A slot whose records all state no range is never split.
    pub fn named(slot: u16, name: impl Into<String>) -> Self {
        Self {
            slot,
            name: name.into(),
            range: None,
        }
    }

    /// The slot the record names.
    pub fn slot(&self) -> u16 {
        self.slot
    }

    /// The name the record states, as text.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The bytecode range the record covers, when it states one.
    pub fn range(&self) -> Option<(u32, u32)> {
        self.range
    }
}

/// One variable one local slot holds (P3 3.4).
///
/// A slot is **one** variable unless the debug table states that it carried two of them over
/// disjoint ranges: the compiler reused one storage location for two source variables whose scopes
/// do not overlap, and the presentation writes two variables with their own names and their own
/// declarations, exactly as the source did.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LocalVariable {
    slot: u16,
    index: u16,
}

impl LocalVariable {
    /// Variable `index` of `slot`, in the order the run's evidence states them.
    pub fn new(slot: u16, index: u16) -> Self {
        Self { slot, index }
    }

    /// The variable a slot holds as a whole: the one index that exists for a slot the evidence does
    /// not split.
    pub fn whole(slot: u16) -> Self {
        Self::new(slot, 0)
    }

    /// The slot.
    pub fn slot(self) -> u16 {
        self.slot
    }

    /// Which of the slot's variables this is: `0` for a slot that is one variable.
    pub fn index(self) -> u16 {
        self.index
    }
}

/// What this run's evidence states about the variables one local slot holds (P3 3.4).
///
/// Produced by [`crate::reuse`] from the body's own records and uses, and read by [`NameTable`]:
/// naming is the *spelling* of the variables that decision states, and a slot this run could not
/// safely split is one unnamed variable rather than one of the two names it carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotEvidence {
    /// No variable of this slot has a name: the body states no record for it, or states records this
    /// run declined to split. The ordinal name is the deterministic fallback either way.
    Unnamed,
    /// One variable for the whole slot, named by the one record that covers it.
    Whole(String),
    /// Several variables, each named by the record the run assigned to it, in variable order.
    Split(Vec<String>),
}

impl SlotEvidence {
    /// The raw name of each variable of this slot, in variable order: one `None` for a slot the
    /// evidence does not name at all.
    fn vars(&self) -> Vec<Option<String>> {
        match self {
            Self::Unnamed => vec![None],
            Self::Whole(name) => vec![Some(name.clone())],
            Self::Split(names) if names.len() > 1 => names.iter().cloned().map(Some).collect(),
            // A "split" with fewer than two variables states no split at all: the slot keeps the one
            // unnamed variable rather than a name that covers only part of it.
            Self::Split(_) => vec![None],
        }
    }
}

/// The name one local variable is written as, with the evidence it came from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedName {
    text: String,
    raw: Option<String>,
    aliased: Option<AliasReason>,
    variable: LocalVariable,
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
        self.variable.slot()
    }

    /// Which of that slot's variables this name is: `0` unless the slot carries several.
    pub fn index(&self) -> u16 {
        self.variable.index()
    }

    /// The variable this name stands for.
    pub fn variable(&self) -> LocalVariable {
        self.variable
    }
}

/// The names of one method's local variables, decided in slot order.
///
/// Built from the declared parameter count (parameter slots are the first ones a body's locals
/// array holds), the number of slots the body has, and the evidence [`crate::reuse`] decided for
/// each slot. Slots above the ones the body states have no name at all: a node that writes one is
/// unrenderable evidence, and the callers of this table treat it as such instead of inventing a
/// spelling for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameTable {
    names: BTreeMap<LocalVariable, RenderedName>,
    aliased: usize,
    invented: usize,
}

impl NameTable {
    /// One row per variable of every slot, decided in slot order — and, inside one slot, in the
    /// order the evidence states its variables — so that collisions resolve deterministically.
    ///
    /// `parameters` is the number of parameter slots (the method's own arity in slots, `this`
    /// included for an instance method when the caller counted it that way), `slots` is how many
    /// local slots the body has, and `evidence` states what this run decided each slot holds. The
    /// table states a name for every variable of every slot the body has: it is the local slots the
    /// body's frames declare, not the debug names it happens to carry, that say which slots a
    /// recovered statement can mention.
    pub fn build(parameters: u16, slots: u16, evidence: &[SlotEvidence]) -> Self {
        let mut table = Self {
            names: BTreeMap::new(),
            aliased: 0,
            invented: 0,
        };
        let slots = slots
            .max(parameters)
            .max(u16::try_from(evidence.len()).unwrap_or(u16::MAX));
        let mut taken: BTreeMap<String, LocalVariable> = BTreeMap::new();
        for slot in 0..slots {
            let vars = evidence
                .get(usize::from(slot))
                .map_or_else(|| SlotEvidence::Unnamed.vars(), SlotEvidence::vars);
            for (index, raw) in vars.into_iter().enumerate() {
                let variable = LocalVariable::new(slot, u16::try_from(index).unwrap_or(u16::MAX));
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
                taken.insert(text.clone(), variable);
                table.names.insert(
                    variable,
                    RenderedName {
                        text,
                        raw,
                        aliased,
                        variable,
                    },
                );
            }
        }
        table
    }

    /// The name of one variable, when the table states one.
    pub fn name(&self, variable: LocalVariable) -> Option<&RenderedName> {
        self.names.get(&variable)
    }

    /// The name of a slot that is **one** variable; `None` when the slot carries several, because
    /// none of them is a name for the whole of it.
    pub fn whole(&self, slot: u16) -> Option<&RenderedName> {
        if self.names.contains_key(&LocalVariable::new(slot, 1)) {
            return None;
        }
        self.names.get(&LocalVariable::whole(slot))
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

    /// The text of one variable's name, when the table states one.
    pub fn text(&self, variable: LocalVariable) -> Option<&str> {
        self.names.get(&variable).map(RenderedName::text)
    }

    /// How many variables could not be written as their evidence spelled them.
    pub fn aliased(&self) -> usize {
        self.aliased
    }

    /// How many variables had no evidence at all and got an ordinal name.
    pub fn invented(&self) -> usize {
        self.invented
    }

    /// Whether any variable's text differs from the spelling its evidence carried.
    ///
    /// This is the one bit the syntax plane reads: an alias means the produced text is not claimed
    /// to spell the input faithfully, whether or not it is legal Java.
    pub fn any_aliased(&self) -> bool {
        self.aliased > 0
    }

    /// Every name the table decided, in slot and variable order.
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
        assert_eq!(table.text(LocalVariable::whole(0)), Some("arg0"));
        assert_eq!(table.text(LocalVariable::whole(1)), Some("arg1"));
        assert_eq!(table.text(LocalVariable::whole(2)), Some("local2"));
        assert_eq!(table.text(LocalVariable::whole(3)), Some("local3"));
        assert_eq!(
            table.text(LocalVariable::whole(4)),
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
        let evidence = vec![
            SlotEvidence::Whole("int".to_string()),
            SlotEvidence::Unnamed,
            SlotEvidence::Whole("count".to_string()),
        ];
        let table = NameTable::build(1, 3, &evidence);
        let first = LocalVariable::whole(0);
        assert_eq!(table.name(first).and_then(RenderedName::raw), Some("int"));
        assert_eq!(table.text(first), Some("int_"));
        assert_eq!(
            table.name(first).and_then(RenderedName::aliased),
            Some(AliasReason::Unspellable)
        );
        assert_eq!(
            table.text(LocalVariable::whole(1)),
            Some("local1"),
            "the slot without evidence"
        );
        assert_eq!(table.text(LocalVariable::whole(2)), Some("count"));
        assert_eq!(table.aliased(), 1);
        assert!(table.any_aliased());
    }

    #[test]
    fn a_collision_resolves_by_slot_order_and_is_an_alias() {
        let evidence = vec![
            SlotEvidence::Whole("value".to_string()),
            SlotEvidence::Whole("value".to_string()),
        ];
        let table = NameTable::build(0, 2, &evidence);
        assert_eq!(table.text(LocalVariable::whole(0)), Some("value"));
        assert_eq!(table.text(LocalVariable::whole(1)), Some("value_2"));
        assert_eq!(
            table
                .name(LocalVariable::whole(1))
                .and_then(RenderedName::aliased),
            Some(AliasReason::Collision)
        );
        assert_eq!(table.aliased(), 1);
    }

    #[test]
    fn one_slot_can_hold_two_variables_and_each_takes_its_own_name() {
        // P3 3.4's `Slot reuse across ranges`: slot 1 carries `c` over one range and `d` over
        // another, so the table states two names for that one slot — and neither is a name for the
        // whole of it, which is what `whole` answers.
        let evidence = vec![
            SlotEvidence::Whole("arg0".to_string()),
            SlotEvidence::Split(vec!["c".to_string(), "d".to_string()]),
        ];
        let table = NameTable::build(1, 2, &evidence);
        assert_eq!(table.text(LocalVariable::new(1, 0)), Some("c"));
        assert_eq!(table.text(LocalVariable::new(1, 1)), Some("d"));
        assert_eq!(table.text(LocalVariable::new(1, 2)), None, "no third one");
        assert_eq!(
            table.whole(1).map(RenderedName::text),
            None,
            "neither name is the whole slot's"
        );
        assert_eq!(table.whole(0).map(RenderedName::text), Some("arg0"));
        assert_eq!(table.invented(), 0, "both names came from the evidence");
        assert!(!table.any_aliased());

        // A split variable whose raw spelling is unspellable, or whose text another variable already
        // took, is aliased by exactly the rules a whole slot's name is: the split adds variables, it
        // does not weaken the guarantees about the text.
        let aliased = NameTable::build(
            1,
            2,
            &[
                SlotEvidence::Whole("c".to_string()),
                SlotEvidence::Split(vec!["int".to_string(), "c".to_string()]),
            ],
        );
        assert_eq!(aliased.text(LocalVariable::new(1, 0)), Some("int_"));
        assert_eq!(
            aliased.text(LocalVariable::new(1, 1)),
            Some("c_2"),
            "the alias of `int` is `int_`, and `c` was already taken by slot 0"
        );
        assert_eq!(aliased.aliased(), 2);
        assert!(aliased.any_aliased());
    }

    #[test]
    fn a_table_that_states_no_split_still_states_one_variable() {
        // The two shapes a "split" that is not one takes: fewer than two names states nothing about
        // the slot, and the slot keeps the one ordinal-named variable it had (P3 3.4 declines to
        // split rather than pick one of the names it cannot place).
        for evidence in [
            SlotEvidence::Unnamed,
            SlotEvidence::Split(Vec::new()),
            SlotEvidence::Split(vec!["only".to_string()]),
        ] {
            let table = NameTable::build(1, 2, &[SlotEvidence::Whole("b".to_string()), evidence]);
            assert_eq!(table.text(LocalVariable::whole(1)), Some("local1"));
            assert_eq!(table.text(LocalVariable::new(1, 1)), None);
            assert_eq!(table.invented(), 1);
        }
    }
}
