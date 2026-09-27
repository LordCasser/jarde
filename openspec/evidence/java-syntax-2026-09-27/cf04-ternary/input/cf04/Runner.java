package cf04;

public class Runner {
    public static void main(String[] args) {
        System.out.println(TernaryCases.positive(2));
        System.out.println(TernaryCases.positive(0));
        System.out.println(TernaryCases.positive(-3));
        System.out.println(TernaryCases.choose(true, false, true));
        System.out.println(TernaryCases.choose(false, false, true));
        System.out.println(TernaryCases.nested(true, false, true));
        System.out.println(TernaryCases.nested(false, false, true));
        System.out.println(new TernaryCases(null, 7).value());
        System.out.println(new TernaryCases("xy", 7).value());
        System.out.println(new TernaryCases("xy", 1, true).value());
        System.out.println(new TernaryCases("xy", 0, true).value());
        for (boolean flag : new boolean[]{true, false}) {
            TernaryCases.calls = 0;
            System.out.println(TernaryCases.effect(flag) + ":" + TernaryCases.calls);
        }
    }
}
