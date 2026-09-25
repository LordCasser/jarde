public class Runner {
    public static void main(String[] args) {
        E[] exposed = E.raw();
        exposed[0] = E.B;
        System.out.println("first=" + E.values()[0] + ",raw=" + E.raw()[0]);
    }
}
