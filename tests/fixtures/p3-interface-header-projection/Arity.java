/// Compiled against `api1/ArityApi.java` (one type parameter). The frozen environment ships the
/// `api2` definition of the same binary name instead, so the class's own `Signature` claim
/// (`ArityApi<Arity>`) contradicts the definition the environment states.
public class Arity implements ArityApi<Arity> {
    public void put(Arity value) {
        System.out.println("put");
    }

    public static void main(String[] args) {
        new Arity().put(new Arity());
    }
}
