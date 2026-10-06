/// The **contradicting** definition the frozen environment ships under the same binary name
/// `ArityApi`, with two type parameters while `Arity`'s own `Signature` states one argument.
/// No javac produces this pair in one compilation; it is a version-skew shape (a class compiled
/// against one revision of an interface, presented against another), assembled from two real
/// javac outputs that are each their own class's whole file.
public interface ArityApi<A, B> {
    void put(A first, B second);
}
