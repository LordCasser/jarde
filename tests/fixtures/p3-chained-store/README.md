# P3 fixture: `dup` followed by two local stores is a chained assignment

`v8/Chain.class` is a real compiled sample: the sibling `Chain.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Chain.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Chain` |
| class-file version | 52.0 (Java 8) |
| bytes | 226 |
| SHA-256 | `0a39e9685ea25b7382f780afce1e56a321c6fece1406ec873aa0d317fe8e2dd9` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_chained_store.rs` (the `chain` text, the two declarations, and the `fill3` control) |

## What the defect is, and what each member is for

`a = b = n` compiles to `iload_0; dup; istore_2; istore_1`: the copy leaves the value on the stack
twice and each store takes one copy. The builder treated every `Operation::Duplicate` as a stated
gap and every value that traced back to one as a value "which produces no expression this subset
writes", so both stores were quoted under the copy's BCI and the member kept a `return local1 +
local2;` that read two locals nothing had ever declared. The presentation was not the class file's
program: it named names the class file does not declare.

The shape the change reads is exactly `dup; store; store` — the two instructions right after the
copy are two stores of **local** slots, and the two of them take the two copies the copy produced.
Where that holds, the value is written **once**: the first store writes the expression the copy
duplicated and the second reads the local the first one filled, which still holds the same value
because nothing runs between them. Writing the expression into both stores would evaluate it twice
(`a = b = f()` would call `f` twice, `a = b = new C()` would allocate twice), which is a program the
bytecode does not have.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `chain(I)I` | `iload_0; dup; istore_2; istore_1; iload_1; iload_2; iadd; ireturn` | BCI 1 quoted (`belongs to no shape this run verified`) and BCIs 2 and 3 quoted as values "from an Duplicate at BCI 1", with `return local1 + local2;` naming two locals nothing declared | `int local2 = arg0; int local1 = local2; return local1 + local2;` — the value written once, and both locals declared before the `return` that reads them |
| `fill3()[I` | `iconst_3; newarray int; dup; iconst_0; iconst_1; iastore; …` | explanation only: BCIs 1, 6, 10 and 14 are "not part of the provable subset", BCIs 3, 7 and 11 are the array initializer's copies, BCI 15 the value behind the `areturn` | unchanged: the copies of the array initializer are followed by an index and a value, not by two local stores, so the shape does not hold and there are **not** three `new int[…]` expressions |

The two members are the shape and the control:

* **`chain`** is the shape the change is about. `javac` stores `b` first (`istore_2`) and `a` second
  (`istore_1`), so the first store the text writes is `local2 = arg0` and the second is `local1 =
  local2` — the two declarations the `return` then reads.
* **`fill3`** is the control that keeps the rule honest in the other direction: an array initializer
  copies the array once per element (`newarray; dup; iconst_0; …; iastore`), and presenting that
  copy as a chained assignment would write `new int[…]` once per element. Its `dup` is followed by a
  `push` and not by two stores, so the shape does not hold, and the member stays exactly as it was.
