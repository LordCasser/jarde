/// The **contradicting** definition the frozen environment ships under the same binary name
/// `NB$Twin`, with two type parameters while `ArityExtends`'s own `Signature` states one
/// argument. Both files are real javac output; the pair is a version-skew shape, not one
/// compilation.
public class NB$Twin<A, B> {
    public A first;
    public B second;
}
