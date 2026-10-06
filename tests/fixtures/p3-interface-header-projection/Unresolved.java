public class Unresolved implements MissingApi<Unresolved> {
    public void put(Unresolved value) {
        System.out.println("put");
    }

    public static void main(String[] args) {
        new Unresolved().put(new Unresolved());
    }
}
