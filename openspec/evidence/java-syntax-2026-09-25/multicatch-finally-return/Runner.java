public final class Runner {
    public static void main(String[] args) {
        for (int mode = 0; mode < 4; mode++) {
            MultiCatchProbe.finallyCalls = 0;
            String plain = MultiCatchProbe.choosePlain(mode);
            String withFinally = MultiCatchProbe.chooseFinally(mode);
            System.out.println(mode + ":" + plain + ":" + withFinally + ":" + MultiCatchProbe.finallyCalls);
        }
    }
}
