package cf04;

public class BasicRunner {
    public static void main(String[] args) {
        System.out.println(TernaryBasic.positive(2));
        System.out.println(TernaryBasic.positive(0));
        System.out.println(TernaryBasic.positive(-3));
        System.out.println(TernaryBasic.choose(true, false, true));
        System.out.println(TernaryBasic.choose(false, false, true));
        System.out.println(new TernaryBasic(null, 7).value());
        System.out.println(new TernaryBasic("xy", 7).value());
        System.out.println(new TernaryBasic("xy", 1, true).value());
        System.out.println(new TernaryBasic("xy", 0, true).value());
        for (boolean flag : new boolean[]{true, false}) {
            TernaryBasic.calls = 0;
            System.out.println(TernaryBasic.effect(flag) + ":" + TernaryBasic.calls);
        }
    }
}
