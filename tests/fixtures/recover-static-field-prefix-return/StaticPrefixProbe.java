public class StaticPrefixProbe {
    static void bumpFrom(int start) {
        FO.i = start;
        int result = FO.bump();
        System.out.println(result + "/" + FO.i);
    }
    static void finallyFrom(int start) {
        FO.i = start;
        int result = FO.evalThenFin();
        System.out.println(result + "/" + FO.i);
    }
    public static void main(String[] args) {
        bumpFrom(-1);
        bumpFrom(7);
        bumpFrom(Integer.MAX_VALUE);
        finallyFrom(7);
        finallyFrom(Integer.MAX_VALUE);
        FO.i = 0;
        FO.main(new String[0]);
    }
}
