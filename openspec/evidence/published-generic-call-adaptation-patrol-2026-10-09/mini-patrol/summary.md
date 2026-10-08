# Exploratory next-call mini-patrol

Four independent Java 8 source/class/JAR inputs were compiled with Corretto 8 `-g`. For each input, original, candidate Jarde and repo-built JADX whole-class source was compiled with independent empty classpath/sourcepath and temporary classes; each `java` execution used only its produced classes. No loose `.class` files remain here. This is exploratory evidence, not acceptance or a design decision.

| Probe | Original | Jarde | JADX | Jarde finding |
|---|---|---|---|---|
| `EmptySink<T>.sink(T)` | compile; reflection sees `T` | compile; reflection sees `Object` | compile; reflection sees `T` | Method signature refused; erased header, with reason `return source is not a proven parameter value or selected member creation`. The unused parameter makes the body compile. |
| `FieldSetter<T>.set(T)` | compiles; marker field identity true | compiles; marker field identity true | compiles; marker field identity true | Method signature projected as `(TT;)V`; field remains `T`. |
| `CallRelay<T>.relay(T)` calling `identity(T)` | compiles; relay returns same marker | compile fails | compiles; relay returns same marker | `identity(T)->T` signature is projected; `relay` is refused and emitted as `Object relay(Object)`. Javac reports `Object cannot be converted to T` at `identity(x)`. |
| `TypedSetter<T>.set(TypedSetter<T>, T)` | compiles; marker field identity true | compiles with unchecked warning; marker field identity true | compiles; marker field identity true | Method signature refused; both parameters erased to raw `TypedSetter` and `Object`. Class/field `T` remains projected. |

All classes retain a generic class declaration `T`. The clear positive case is a void field store: the method parameter and field type are both reconstructed as `T`, and behavior matches. An unused void parameter has no source evidence for `T`, so its parameter erases. A generic relay caller is refused while its callee `identity` is projected, creating a concrete compile failure. A two-parameter void setter is also refused despite the receiver and value field context; raw receiver erasure compiles with an unchecked warning. Full source, diagnostics, byte/hash manifest, and command outputs are beside this summary.
