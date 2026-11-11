package em12;

public class Case extends Parent {
    class Member {
        String call(Arg value) {
            return Case.super.pick(value);
        }
    }

    public static void main(String[] args) {
        Case outer = new Case();
        java.io.PrintStream out = System.out;
        java.util.Objects.requireNonNull(outer);
        out.println(outer.new Member().call(null));
    }
}
