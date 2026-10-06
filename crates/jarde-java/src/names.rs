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
//! One slot is neither named from evidence nor from an ordinal: local slot 0 of a member that is not
//! `static` holds the **receiver** (JVMS 4.10.1.9), which is written [`RECEIVER`] wherever the body
//! reads it, whatever the debug table states for that slot and whether or not it states anything at
//! all. A caller states that its member is one of those with
//! [`NameTable::build_with_receiver`], from the member's own `ACC_STATIC` fact — never from a name
//! the class file happens to carry.
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

/// The text the receiver of a member's body is written as (JVMS 4.10.1.9).
///
/// Local slot 0 of a member that is not `static` holds the receiver, and the receiver is written as
/// the keyword in every position the body reads that slot in — a field read `this.f`, a call
/// `this.m()`, an argument `f(this)`. The spelling is a fact about the **slot** and not a name read
/// out of the class: a `LocalVariableTable` may call slot 0 `this`, `this$0`, `self` or nothing at
/// all, and the slot is the receiver under every one of those. It is the one spelling this module
/// writes that is a Java keyword, which is what makes it collision-free: no local or parameter a
/// body declares can carry it, because Java forbids declaring it.
pub const RECEIVER: &str = "this";

/// Whether `text` is an identifier this layer is willing to write.
///
/// The grammar is Java's (JLS §3.8): a *Java letter* first, then Java letters and digits. What
/// "letter" and "digit" mean is Unicode's and not ASCII's, and that is not decoration: the names
/// this function judges are JVMS §4.7.7 UTF-8, and a class file that declares a field `变量` and
/// reads it back names **one** string. An answer of `false` for it would spell the declaration `__`
/// (through [`alias_for`]) while the same run's own body statements write the pool's `变量` — one
/// name with two spellings, which is the collision this function exists to prevent and a text no
/// compiler accepts. So the predicate is the JLS one, decided from the Unicode properties `std`
/// states, with Java's two additions kept exactly as they were: `$` and `_` are letters, and the
/// keywords (`true`/`false`/`null` and the reserved `const`/`goto` included) are not identifiers.
///
/// The relation to `Character.isJavaIdentifierStart`/`Character.isJavaIdentifierPart` is the JLS
/// rule as far as `std`'s Unicode properties state it, and the difference runs both ways:
///
/// * accepted, where Java accepts them: the letters (`Alphabetic` is Unicode's letters plus the
///   numeric letters `Ⅷ` its start rule admits too) and the digits `is_alphanumeric` adds
///   (`变量`, `café`, `x١٢٣` — a leading digit starts no identifier for either of us);
/// * accepted, where Java does not: the **other numeric** characters — `is_alphanumeric` is
///   `Alphabetic || Numeric`, and `Numeric` covers `No` (`²`, `①`) on top of the decimal digits
///   `Nd` and the numeric letters `Nl` Java admits — and a mark Unicode counts as `Alphabetic`
///   (a Devanagari vowel sign) in the start position, where Java asks for a letter. Both are the
///   superset side: the layer spells the pool's name instead of hiding it behind an alias;
/// * refused, where Java accepts them: the characters Java admits under its *other* categories —
///   a currency symbol other than `$` (`€`), connecting punctuation other than `_`, a combining
///   mark, an identifier-ignorable format character (U+200B). `std` states no Unicode general
///   category, and the alternative to refusing would be admitting whatever is neither space nor
///   control, which would write `。` and `·` — names Java cannot spell either — into identifier
///   positions. Such a name keeps [`alias_for`] and the marker that states its raw spelling, which
///   is the presentation every revision before this one gave it;
/// * the ASCII half is answered byte-wise before any of that ([`ascii_identifier`]), so every name
///   of every corpus written before non-ASCII names were admitted is decided by the exact table it
///   was decided by: an ASCII identifier's rendering cannot move by one byte, and ASCII punctuation
///   (`-`, `.`, `/`, `;`) still never takes an identifier position.
pub fn is_java_identifier(text: &str) -> bool {
    if text.is_empty() || JAVA_KEYWORDS.contains(&text) {
        return false;
    }
    if text.is_ascii() {
        return ascii_identifier(text);
    }
    let mut characters = text.chars();
    let first = characters
        .next()
        .expect("non-empty text was established above");
    (first.is_alphabetic() || first == '_' || first == '$')
        && characters
            .all(|character| character.is_alphanumeric() || character == '_' || character == '$')
}

/// The ASCII half of [`is_java_identifier`], byte-wise: `[A-Za-z_$][A-Za-z0-9_$]*`.
///
/// This is the whole function the earlier revisions were, and it is kept as its own reading so that
/// the ASCII answer is visibly the table it always was rather than the `char` properties standing
/// in for it.
fn ascii_identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    let start = first == b'_' || first == b'$' || first.is_ascii_alphabetic();
    start && bytes.all(|byte| byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric())
}

/// The Java source spelling of one class-file nested name, as the **reference** a body of
/// `current_class` writes in a type position.
///
/// A binary name spells nesting with `$` (`V1$Op`), and Java source cannot write that token in a
/// type position. This function is the presentation layer of the **cross-class** half of that
/// rule: a nested name whose top-level owner is a *different* class than the one whose text is
/// being written is spelled with the nesting dotted (`Other.Inner` from `VN1`, `p.A.B.C` for a
/// deeper chain), which is legal source wherever the owner is resolvable — the same situation a
/// package-level reference is in, and deliberately not this layer's problem.
///
/// The **self-nested** half — a reference to a member of the class's own top-level owner
/// (`V1$Op` inside `V1`, `N1$Numbers$NumString` inside `N1$Numbers`) — is **not** converted here,
/// and that is a decision, not an omission. A recovery text states such a member's declaration
/// only when a family projection folds it into the same text (the nested-enum fold, the proved
/// static member writer); the member is otherwise presented as its own physical unit under the
/// pool's `$` name, and a project built from those units resolves exactly that name. The simple
/// source spelling (`Op`) is therefore written by the projection that renders the nested
/// declaration, never by this generic rule — the two spellings must not diverge from the
/// declaration the text actually carries.
///
/// `current_class` is the class whose text the reference is written into, in either the internal
/// (`p/Outer`) or the source (`p.Outer`) spelling; its top-level owner is everything before the
/// first `$`. The conversion is refused, and the name returned exactly as it is, in every case
/// this layer cannot spell better than the bytes: a name with no `$`, a name whose `$`-separated
/// tail has a segment that is not a Java identifier (javac's synthetic families: the anonymous
/// `X$1`, the local `X$1Local`), a self-nested name as defined above, and `current_class` of
/// `None` — a run that stated no declaring class has no owner to compare against.
///
/// An array suffix (`V1$Op[]`) is not part of the nesting: it is split off, the element name is
/// spelled, and the suffix is written back after it.
pub fn nested_reference_spelling<'n>(
    name: &'n str,
    current_class: Option<&str>,
) -> std::borrow::Cow<'n, str> {
    use std::borrow::Cow;
    // The fast path is also the guarantee the no-`$` families keep byte-identical text: platform
    // names, simple names and already-dotted names never allocate and never change.
    if !name.contains('$') {
        return Cow::Borrowed(name);
    }
    let Some(current) = current_class else {
        return Cow::Borrowed(name);
    };
    let current = current.replace('/', ".");
    // The `[]` of an array type is a suffix of the whole spelling, so the element name is
    // everything before its first occurrence and the suffix is the rest of the text unchanged.
    let (element, suffix) = match name.split_once("[]") {
        Some((element, _)) => (element, &name[element.len()..]),
        None => (name, ""),
    };
    let Some((head, tail)) = element.split_once('$') else {
        return Cow::Borrowed(name);
    };
    // Both sides of the split must be names Java can write: the head is a package prefix plus a
    // top-level class (`p.Outer`, `V1`), and every segment of the tail is a nested member's own
    // name. A name that fails this is one the pool states in a form source cannot improve on —
    // the synthetic families above, or a hostile spelling — and it keeps the pool's form.
    if head.is_empty()
        || !head.split('.').all(is_java_identifier)
        || !tail.split('$').all(is_java_identifier)
    {
        return Cow::Borrowed(name);
    }
    // The self-nested boundary: the name's top-level owner is the class whose text this is, so
    // the member's declaration belongs to this unit's own family and only a fold projection may
    // spell it as source nesting (see this function's doc). Everything else keeps the pool name.
    let owner = current.split('$').next().unwrap_or(&current);
    if head == owner {
        return Cow::Borrowed(name);
    }
    let nested = tail.split('$').collect::<Vec<&str>>();
    let mut spelled = format!("{head}.{}", nested.join("."));
    spelled.push_str(suffix);
    Cow::Owned(spelled)
}

/// The source spelling of one nested class name, as a **member of the class whose own
/// `InnerClasses` attribute is `members`** writes it.
///
/// This is [`nested_reference_spelling`] behind the one piece of evidence the rule cannot invent:
/// a `$` alone does not state a nesting relation, because `$` is a legal character of a **top
/// level** class's own name (`Named$Top` is one class, not `Named`'s member `Top`), and JVMS
/// §4.7.6 states the fact that does — a class file that references a member class carries an
/// `InnerClasses` row naming it. `members` is that row set of the class whose text is being
/// written, each name in the source spelling this module compares with (`V1$Op`, `p.A$B$C`); a
/// name that is not one of them keeps the pool's own spelling exactly, and so does every name
/// when the run stated no rows at all.
pub fn nested_member_reference_spelling<'n>(
    name: &'n str,
    current_class: Option<&str>,
    members: Option<&[String]>,
) -> std::borrow::Cow<'n, str> {
    let element = match name.split_once("[]") {
        Some((element, _)) => element,
        None => name,
    };
    let stated = members.is_some_and(|members| members.iter().any(|member| member == element));
    if !stated {
        return std::borrow::Cow::Borrowed(name);
    }
    nested_reference_spelling(name, current_class)
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
    descriptor: Option<Vec<u8>>,
    generic_signature: Option<Vec<u8>>,
}

impl DebugLocal {
    /// A record naming one slot over the bytecode range `[start_bci, end_bci)`.
    pub fn over(slot: u16, name: impl Into<String>, start_bci: u32, end_bci: u32) -> Self {
        Self {
            slot,
            name: name.into(),
            range: Some((start_bci, end_bci)),
            descriptor: None,
            generic_signature: None,
        }
    }

    /// A name for one slot with no range stated: what an entry point that was handed one name per
    /// slot states. A slot whose records all state no range is never split.
    pub fn named(slot: u16, name: impl Into<String>) -> Self {
        Self {
            slot,
            name: name.into(),
            range: None,
            descriptor: None,
            generic_signature: None,
        }
    }

    /// Adds the exact LVT erasure and uniquely matched LVTT signature from one reader run.
    pub fn with_type_metadata(
        mut self,
        descriptor: impl Into<Vec<u8>>,
        generic_signature: impl Into<Vec<u8>>,
    ) -> Self {
        self.descriptor = Some(descriptor.into());
        self.generic_signature = Some(generic_signature.into());
        self
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

    /// The LVT descriptor bytes when this record has a unique matched generic signature.
    pub fn descriptor(&self) -> Option<&[u8]> {
        self.descriptor.as_deref()
    }

    /// The raw LVTT Signature bytes when this record has a unique exact LVT match.
    pub fn generic_signature(&self) -> Option<&[u8]> {
        self.generic_signature.as_deref()
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
    /// Several variables in one slot, with a debug name only where a record belongs to that
    /// variable. An inferred lifetime can precede the only LVT record for its slot.
    Split(Vec<Option<String>>),
}

impl SlotEvidence {
    /// The raw name of each variable of this slot, in variable order: one `None` for a slot the
    /// evidence does not name at all.
    fn vars(&self) -> Vec<Option<String>> {
        match self {
            Self::Unnamed => vec![None],
            Self::Whole(name) => vec![Some(name.clone())],
            Self::Split(names) if names.len() > 1 => names.clone(),
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
    reserved: BTreeSet<String>,
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
    ///
    /// Slot 0 of this layout is a **parameter**, so this is the constructor for a `static` member —
    /// or for a caller that stated no member flags. A body whose slot 0 holds the receiver takes
    /// [`Self::build_with_receiver`] instead.
    pub fn build(parameters: u16, slots: u16, evidence: &[SlotEvidence]) -> Self {
        Self::decide(parameters, slots, evidence, false, &BTreeSet::new())
    }

    /// The same table for a body whose slot 0 holds the **receiver** (JVMS 4.10.1.9).
    ///
    /// Slot 0 is then written as [`RECEIVER`] wherever the body reads it, whatever the evidence
    /// states for the slot: a debug name that happens to be `this` must not be taken for a keyword
    /// to alias into `this_`, and a body with no debug table must get `this` rather than the ordinal
    /// name `arg0`. Which member takes a receiver is the caller's own fact — the member's
    /// `ACC_STATIC` bit, read from the member declaration the same run already holds — and this
    /// module is *told* the answer instead of reading a name or a slot ordinal to find it.
    ///
    /// Every other slot keeps the rules [`Self::build`] states for it.
    pub fn build_with_receiver(parameters: u16, slots: u16, evidence: &[SlotEvidence]) -> Self {
        Self::decide(parameters, slots, evidence, true, &BTreeSet::new())
    }

    /// The same naming walk with names already claimed by proven simple field writes. Those names
    /// are reserved before local evidence is considered, so a local cannot shadow a field that the
    /// emitter will write without a qualifier.
    pub(crate) fn build_with_reserved(
        parameters: u16,
        slots: u16,
        evidence: &[SlotEvidence],
        reserved: &BTreeSet<String>,
    ) -> Self {
        Self::decide(parameters, slots, evidence, false, reserved)
    }

    /// Receiver variant of [`Self::build_with_reserved`].
    pub(crate) fn build_with_receiver_and_reserved(
        parameters: u16,
        slots: u16,
        evidence: &[SlotEvidence],
        reserved: &BTreeSet<String>,
    ) -> Self {
        Self::decide(parameters, slots, evidence, true, reserved)
    }

    /// The one naming walk both constructors above state their input to.
    fn decide(
        parameters: u16,
        slots: u16,
        evidence: &[SlotEvidence],
        receiver: bool,
        reserved: &BTreeSet<String>,
    ) -> Self {
        let mut table = Self {
            names: BTreeMap::new(),
            reserved: reserved.clone(),
            aliased: 0,
            invented: 0,
        };
        let slots = slots
            .max(parameters)
            .max(u16::try_from(evidence.len()).unwrap_or(u16::MAX));
        let mut taken = reserved.clone();
        for slot in 0..slots {
            // The receiver is the whole of slot 0: every variable the evidence states for that slot
            // is a read of the same instance, so none of them is spelled as the evidence names it.
            let receiver_slot = receiver && slot == 0;
            let vars = evidence
                .get(usize::from(slot))
                .map_or_else(|| SlotEvidence::Unnamed.vars(), SlotEvidence::vars);
            for (index, raw) in vars.into_iter().enumerate() {
                let variable = LocalVariable::new(slot, u16::try_from(index).unwrap_or(u16::MAX));
                let (text, aliased) = match (receiver_slot, &raw) {
                    (true, _) => (RECEIVER.to_string(), None),
                    (false, None) => {
                        table.invented += 1;
                        (invented_name(slot, parameters), None)
                    }
                    (false, Some(raw)) if is_java_identifier(raw) => (raw.clone(), None),
                    (false, Some(raw)) => (alias_for(raw), Some(AliasReason::Unspellable)),
                };
                let (text, aliased) = match taken.contains(&text) {
                    false => (text, aliased),
                    true => {
                        let mut suffix = 2u32;
                        loop {
                            let candidate = format!("{text}_{suffix}");
                            if !taken.contains(&candidate) {
                                break (candidate, Some(AliasReason::Collision));
                            }
                            suffix += 1;
                        }
                    }
                };
                if aliased.is_some() {
                    table.aliased += 1;
                }
                taken.insert(text.clone());
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
        self.free_name_with(base, || Ok::<(), ()>(()))
            .expect("the non-budgeted name walk cannot fail")
    }

    /// The same deterministic search with one callback at every bounded unit of work: each
    /// reserved/local spelling copied into the temporary set, and each candidate checked before
    /// it is accepted or extended. Recovery paths that own a [`Budget`](jarde_reader::budget::Budget)
    /// use this hook so a large collision set is charged and cancellable inside the name walk,
    /// rather than only once around the caller's suffix loop.
    pub(crate) fn free_name_with<E>(
        &self,
        base: &str,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<String, E> {
        let taken: BTreeSet<&str> = self
            .reserved
            .iter()
            .map(|name| {
                visit()?;
                Ok(name.as_str())
            })
            .collect::<Result<_, E>>()?;
        let mut taken = taken;
        for name in self.names.values() {
            visit()?;
            taken.insert(name.text.as_str());
        }
        let mut candidate = base.to_string();
        loop {
            visit()?;
            if !taken.contains(candidate.as_str()) && is_java_identifier(&candidate) {
                return Ok(candidate);
            }
            candidate.push('_');
        }
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
    fn cross_class_nested_references_spell_dotted_and_self_nested_stays_pool() {
        // Cross-class: the tail is dotted onto the head, package included, however deep.
        assert_eq!(
            nested_reference_spelling("Other$Inner", Some("V1")).as_ref(),
            "Other.Inner"
        );
        assert_eq!(
            nested_reference_spelling("p.A$B$C", Some("p/VN2")).as_ref(),
            "p.A.B.C"
        );
        assert_eq!(
            nested_reference_spelling("p.A$B$C", Some("VN2")).as_ref(),
            "p.A.B.C",
            "the owner comparison is whole-name: a same-named class in another package is another class"
        );
        // The head is compared as a whole: a same-named class in another package is another class.
        assert_eq!(
            nested_reference_spelling("q.Outer$Inner", Some("p/Outer")).as_ref(),
            "q.Outer.Inner"
        );
        // An array's element carries the nesting; the brackets are written back.
        assert_eq!(
            nested_reference_spelling("Other$Inner[]", Some("V1")).as_ref(),
            "Other.Inner[]"
        );
        // Self-nested — the name's top-level owner is the class whose text this is — keeps the
        // pool spelling: only a family projection that declares the member in this text may spell
        // it as source nesting, and this generic rule never does.
        assert_eq!(
            nested_reference_spelling("V1$Op", Some("V1")).as_ref(),
            "V1$Op",
            "the enum fold owns the simple spelling of its own child"
        );
        assert_eq!(
            nested_reference_spelling("V1$Op", Some("V1$Op")).as_ref(),
            "V1$Op",
            "a member's own text keeps the pool spelling of itself and its siblings"
        );
        assert_eq!(
            nested_reference_spelling("p.VN2$Mid$Leaf", Some("p/VN2")).as_ref(),
            "p.VN2$Mid$Leaf",
            "a deep self-nested chain keeps the pool spelling"
        );
        assert_eq!(
            nested_reference_spelling("N1$Numbers$NumString", Some("N1$Numbers")).as_ref(),
            "N1$Numbers$NumString",
            "a grandchild keeps the pool spelling inside its owner's own text"
        );
    }

    #[test]
    fn names_source_already_spells_or_cannot_improve_stay_verbatim() {
        for name in ["java.util.List", "Other", "V1.Op", "int", ""] {
            assert_eq!(
                nested_reference_spelling(name, Some("V1")).as_ref(),
                name,
                "`{name}` is already a source spelling"
            );
        }
        // The synthetic families: an anonymous capture class and javac's local-class spellings,
        // including the one whose tail would pass an identifier check if the head were not checked.
        for name in [
            "C1$1",
            "C1$12",
            "More17$1",
            "Outer$1Local",
            "A$1$B",
            "$Op",
            "A$",
            "p.$Op",
        ] {
            assert_eq!(
                nested_reference_spelling(name, Some("A")).as_ref(),
                name,
                "`{name}` keeps the pool's own spelling"
            );
        }
        // No class to nest against: the pool spelling, never a guess.
        assert_eq!(nested_reference_spelling("V1$Op", None).as_ref(), "V1$Op");
        // A nested class referencing itself whole: the reference's top-level owner is the owner
        // of the text itself, so it is self-nested and keeps the pool spelling.
        assert_eq!(
            nested_reference_spelling("V1$Op", Some("V1$Op")).as_ref(),
            "V1$Op"
        );
    }

    #[test]
    fn a_dollar_top_level_class_is_a_member_of_nothing_the_rows_do_not_state() {
        let members = vec!["V1$Op".to_owned(), "p.A$B$C".to_owned(), "V1$1".to_owned()];
        // A top-level class whose own name carries `$` is not nested, and no rule of this module
        // may turn it into a nesting: the pool spelling is the only faithful one.
        assert_eq!(
            nested_member_reference_spelling(
                "Named$Top",
                Some("StaticMemberBasic"),
                Some(&members)
            )
            .as_ref(),
            "Named$Top"
        );
        // A cross-class name the class's own rows state converts; the same name without that
        // evidence keeps the pool spelling, and so does a run that stated no rows at all.
        assert_eq!(
            nested_member_reference_spelling("p.A$B$C", Some("p/VN2"), Some(&members)).as_ref(),
            "p.A.B.C"
        );
        assert_eq!(
            nested_member_reference_spelling("p.A$B$C", Some("p/VN2"), Some(&[])).as_ref(),
            "p.A$B$C"
        );
        assert_eq!(
            nested_member_reference_spelling("p.A$B$C", Some("p/VN2"), None).as_ref(),
            "p.A$B$C"
        );
        // The row set gates the whole name, arrays included by their element.
        assert_eq!(
            nested_member_reference_spelling("p.A$B$C[]", Some("p/VN2"), Some(&members)).as_ref(),
            "p.A.B.C[]"
        );
    }

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

    /// The identifier table of change `recover-unicode-identifiers`, as the two halves it is: the
    /// ASCII table every corpus before that change was written with, and the JLS reading of the
    /// Unicode properties beside it.
    ///
    /// The contrast is the point of the change being one predicate and not two sources: the same
    /// function the declaration path asks (`written_name` in the presentation) and the reference
    /// path asks (the naming table), so the spellings of one name cannot disagree any more.
    #[test]
    fn the_identifier_table_is_javas_beside_a_byte_identical_ascii_half() {
        // The ASCII half, exactly as every revision before this one decided it.
        for accepted in [
            "a", "A", "$value_2", "_x", "变量", "a变量", "_变量", "变量1",
        ] {
            assert!(is_java_identifier(accepted), "{accepted} is an identifier");
        }
        for refused in [
            "", "1x", "a-b", "a.b", "a/b", "a b", "int", "while", "true", "null", "_", "goto",
        ] {
            assert!(!is_java_identifier(refused), "{refused} is not one");
        }
        // The Unicode half: the letters, digits and `$`/`_` the JLS admits — and the marks,
        // currency symbols and punctuation it does not, which keep their alias and marker.
        assert!(is_java_identifier("café"));
        assert!(is_java_identifier("变量"));
        assert!(is_java_identifier("内部类"));
        assert!(is_java_identifier("方法2"));
        assert!(is_java_identifier("x١٢٣"), "a decimal digit is a part");
        assert!(!is_java_identifier("变量。"), "U+3002 is punctuation");
        assert!(!is_java_identifier("变量·量"), "U+00B7 is punctuation");
        assert!(!is_java_identifier("3变量"), "a digit starts no identifier");
        assert!(!is_java_identifier("١٢٣x"), "not even a decimal one");
        assert!(!is_java_identifier("😀"));
        // The alias generator itself is byte-wise unchanged: a name it is still handed (the
        // punctuation one above) aliases exactly as ASCII-only revisions aliased it.
        assert_eq!(alias_for("变量。"), "___");
        assert_eq!(
            alias_for("😀"),
            "__",
            "the `_` keyword eats the second underscore"
        );
        assert_eq!(
            alias_for("变量。"),
            alias_for("变量。"),
            "pure in the spelling"
        );
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
    fn the_receiver_is_spelled_by_its_identity_and_not_by_the_name_the_table_states() {
        // A `LocalVariableTable` that names slot 0 `this` — what javac writes for an instance
        // member — is not a name to alias: the slot is the receiver (JVMS 4.10.1.9) and the keyword
        // is how it is written. The evidence stays readable as evidence.
        let named = NameTable::build_with_receiver(
            3,
            3,
            &[
                SlotEvidence::Whole("this".to_string()),
                SlotEvidence::Whole("left".to_string()),
                SlotEvidence::Whole("right".to_string()),
            ],
        );
        let receiver = LocalVariable::whole(0);
        assert_eq!(named.text(receiver), Some(RECEIVER));
        assert_eq!(
            named.name(receiver).and_then(RenderedName::raw),
            Some("this"),
            "the record still states what the table said"
        );
        assert_eq!(
            named.name(receiver).and_then(RenderedName::aliased),
            None,
            "the receiver is not an alias of the name it was given"
        );
        assert!(!named.any_aliased());
        assert_eq!(named.invented(), 0);
        // Every other slot keeps the rule it had.
        assert_eq!(named.text(LocalVariable::whole(1)), Some("left"));
        assert_eq!(named.text(LocalVariable::whole(2)), Some("right"));

        // The spelling does not come from the table at all: `this$0`, `self` and no record at all
        // are one spelling.
        for evidence in [
            SlotEvidence::Whole("this$0".to_string()),
            SlotEvidence::Whole("self".to_string()),
            SlotEvidence::Unnamed,
        ] {
            let table = NameTable::build_with_receiver(2, 2, &[evidence, SlotEvidence::Unnamed]);
            assert_eq!(table.text(receiver), Some(RECEIVER));
            assert_eq!(
                table.text(LocalVariable::whole(1)),
                Some("arg1"),
                "the parameter above the receiver keeps the ordinal rule"
            );
            assert_eq!(
                table.invented(),
                1,
                "only the parameter without evidence was named by its ordinal"
            );
            assert!(!table.any_aliased());
        }
    }

    #[test]
    fn a_static_members_slot_zero_is_a_parameter_and_keeps_its_own_naming() {
        // The counterexample: slot 0 of a `static` member is its first parameter (or a local), so
        // the ordinal rule and the keyword alias apply to it exactly as to every other slot — the
        // receiver spelling is not a decoration of slot 0.
        let unnamed = NameTable::build(1, 2, &[]);
        assert_eq!(unnamed.text(LocalVariable::whole(0)), Some("arg0"));
        assert_ne!(unnamed.text(LocalVariable::whole(0)), Some(RECEIVER));

        let named = NameTable::build(2, 2, &[SlotEvidence::Whole("seed".to_string())]);
        assert_eq!(named.text(LocalVariable::whole(0)), Some("seed"));
        assert_eq!(named.text(LocalVariable::whole(1)), Some("arg1"));

        // A debug table that names a `static` method's slot 0 `this` states a name Java cannot
        // spell, and it takes the alias path it took before: the fact that decides the receiver is
        // the member's own `ACC_STATIC`, never the name in front of it.
        let aliased = NameTable::build(1, 1, &[SlotEvidence::Whole("this".to_string())]);
        assert_eq!(aliased.text(LocalVariable::whole(0)), Some("this_"));
        assert_eq!(aliased.aliased(), 1);
        assert!(aliased.any_aliased());
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
    fn proven_field_names_are_reserved_for_locals_and_free_names() {
        let reserved = ["local0", "local0_2"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let table = NameTable::build_with_reserved(0, 1, &[], &reserved);
        assert_eq!(table.text(LocalVariable::whole(0)), Some("local0_3"));
        assert_eq!(table.free_name("local0"), "local0_");
        assert_eq!(table.free_name("local0_2"), "local0_2_");
    }

    #[test]
    fn free_name_with_visits_every_reserved_local_and_candidate() {
        let reserved = ["local0", "local0_2"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let table = NameTable::build_with_reserved(0, 1, &[], &reserved);
        let mut visits = 0;
        let name = table
            .free_name_with("local0", || {
                visits += 1;
                Ok::<(), ()>(())
            })
            .expect("the test callback never stops");
        assert_eq!(name, "local0_");
        assert_eq!(visits, 5, "two reserved, one local, and two candidates");

        let mut stopped_at = 0;
        let result = table.free_name_with("local0", || {
            stopped_at += 1;
            if stopped_at == 3 {
                Err("cancelled")
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err("cancelled"));
        assert_eq!(
            stopped_at, 3,
            "the callback stops the name walk before a candidate"
        );
    }

    #[test]
    fn one_slot_can_hold_two_variables_and_each_takes_its_own_name() {
        // P3 3.4's `Slot reuse across ranges`: slot 1 carries `c` over one range and `d` over
        // another, so the table states two names for that one slot — and neither is a name for the
        // whole of it, which is what `whole` answers.
        let evidence = vec![
            SlotEvidence::Whole("arg0".to_string()),
            SlotEvidence::Split(vec![Some("c".to_string()), Some("d".to_string())]),
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
                SlotEvidence::Split(vec![Some("int".to_string()), Some("c".to_string())]),
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
            SlotEvidence::Split(vec![Some("only".to_string())]),
        ] {
            let table = NameTable::build(1, 2, &[SlotEvidence::Whole("b".to_string()), evidence]);
            assert_eq!(table.text(LocalVariable::whole(1)), Some("local1"));
            assert_eq!(table.text(LocalVariable::new(1, 1)), None);
            assert_eq!(table.invented(), 1);
        }
    }
}
