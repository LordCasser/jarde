#!/usr/bin/env python3
"""Temporarily instrument `project_static_fold_owner_texts` so each token prints its covering
bcis, those instructions' pool-entry kinds, and the anchor route finally taken (direct / handler /
producer / refused). Usage: apply_probe.py <src/facade.rs>

Prints one `SIFDBG ...` line to stderr per token examined. Reverted with `git checkout`.
"""
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
source = path.read_text()

COVER_DECL = """                let probe_covers: Vec<String> = source_bcis
                    .iter()
                    .copied()
                    .map(|bci| {
                        let instruction = code.instructions.iter().find(|insn| insn.bci == bci);
                        let index = instruction.and_then(|insn| insn.constant_pool_index);
                        let kind = index.and_then(|index| {
                            jarde_reader::classfile::cp_entry(analyzed.ir().constant_pool(), index)
                                .ok()
                        });
                        let kind = kind.map(|entry| match &entry.kind {
                            jarde_reader::classfile::CpEntryKind::Class { name, .. } => {
                                format!("Class[{}]", String::from_utf8_lossy(&name.0))
                            }
                            jarde_reader::classfile::CpEntryKind::FieldRef { owner, name, .. } => {
                                format!(
                                    "FieldRef[{}#{}]",
                                    String::from_utf8_lossy(&owner.0),
                                    String::from_utf8_lossy(&name.0)
                                )
                            }
                            jarde_reader::classfile::CpEntryKind::MethodRef { owner, name, .. } => {
                                format!(
                                    "MethodRef[{}#{}]",
                                    String::from_utf8_lossy(&owner.0),
                                    String::from_utf8_lossy(&name.0)
                                )
                            }
                            jarde_reader::classfile::CpEntryKind::InterfaceMethodRef {
                                owner,
                                name,
                                ..
                            } => format!(
                                "InterfaceMethodRef[{}#{}]",
                                String::from_utf8_lossy(&owner.0),
                                String::from_utf8_lossy(&name.0)
                            ),
                            other => format!("other({other:?})"),
                        });
                        format!(
                            "bci={bci} op={:#04x} cp={}",
                            instruction.map_or(0, |insn| insn.opcode),
                            kind.unwrap_or_else(|| "none".to_owned())
                        )
                    })
                    .collect();
                let probe_token =
                    String::from_utf8_lossy(&body_text.as_bytes()[start..end]).into_owned();
                let probe_target = String::from_utf8_lossy(&target.binary).into_owned();
"""

pairs = [
    (
        """                let anchor_bci = if let Some(bci) = matching.first().copied() {
                    bci
                } else {""",
        COVER_DECL
        + """                let anchor_bci = if let Some(bci) = matching.first().copied() {
                    eprintln!(
                        "SIFDBG direct token={probe_token:?} target={probe_target} anchor_bci={bci} covers={}",
                        probe_covers.join(" | ")
                    );
                    bci
                } else {""",
    ),
    (
        """                    if let Some(handler) = handler {
                        handler.handler_bci
                    } else {""",
        """                    if let Some(handler) = handler {
                        eprintln!(
                            "SIFDBG handler token={probe_token:?} target={probe_target} anchor_bci={} covers={}",
                            handler.handler_bci,
                            probe_covers.join(" | ")
                        );
                        handler.handler_bci
                    } else {""",
    ),
    (
        """                        let Some(anchor) = stored else {
                            return Ok(Err(format!(
                                "static fold token in method {} is not tied to one proved class reference",
                                method.item.index
                            )));
                        };
                        anchor""",
        """                        let Some(anchor) = stored else {
                            eprintln!(
                                "SIFDBG refused token={probe_token:?} target={probe_target} method={} covers={}",
                                method.item.index,
                                probe_covers.join(" | ")
                            );
                            return Ok(Err(format!(
                                "static fold token in method {} is not tied to one proved class reference",
                                method.item.index
                            )));
                        };
                        eprintln!(
                            "SIFDBG producer token={probe_token:?} target={probe_target} anchor_bci={anchor} covers={}",
                            probe_covers.join(" | ")
                        );
                        anchor""",
    ),
]

for old, new in pairs:
    if source.count(old) != 1:
        raise SystemExit(f"pattern not unique ({source.count(old)}): {old.splitlines()[0]!r}")
    source = source.replace(old, new)
path.write_text(source)
print("probe applied")