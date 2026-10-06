/// The interaction boundary this change records: a body that calls the erased contract on a
/// receiver whose recovered static type is the **implementing class**. The bytecode call is
/// `Comparable.compareTo:(Ljava/lang/Object;)I`, so the recovered text spells the erased cast
/// (`compareTo((java.lang.Object) …)`) — a call only the visible bridge could answer. With the
/// parameterized header the bridge hides and javac regenerates it, but the *body's* spelled call
/// still names the erased parameter, so this one body does not compile until the
/// invocation-argument-typing domain re-types such call sites (a separate slice's debt; the
/// class text carries the standard per-member marker).
public class ErasedCall implements java.lang.Comparable<ErasedCall> {
    public int compareTo(ErasedCall other) {
        return 0;
    }

    public static void main(String[] args) {
        java.lang.Comparable<ErasedCall> contract = new ErasedCall();
        System.out.println(contract.compareTo(new ErasedCall()));
    }
}
