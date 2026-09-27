package em22;

public class Runner {
    public static void main(String[] args) {
        Arithmetic a = new Arithmetic();
        System.out.println(a.multiply(4) + ":" + a.sum(1, 2, 3));
        System.out.println(a.subtract(20, 8, 2) + ":" + a.divide(20, 8, 2));
        System.out.println(a.or(false, true, false) + ":" + a.and(true, true, false));
        System.out.println(a.notInt(0) + ":" + a.notLong(0L));
        System.out.println(a.flip(true) + ":" + a.flip(false));
        System.out.println(a.eagerOr() + ":" + a.calls());
        System.out.println(a.flipCall() + ":" + a.calls());
        System.out.println(a.sameCall() + ":" + a.calls());
    }
}
