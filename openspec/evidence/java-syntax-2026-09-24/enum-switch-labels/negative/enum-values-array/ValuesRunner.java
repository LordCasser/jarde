public class ValuesRunner {
    public static void main(String[] args) {
        Hue[] values = Hue.values();
        if (values == null) {
            System.out.println("null");
        } else {
            System.out.println(values[0] + "," + values[1] + "," + values[2]);
        }
        try {
            System.out.println(EnumSwitchSubject.choose(Hue.RED) + ","
                    + EnumSwitchSubject.choose(Hue.BLUE) + ","
                    + EnumSwitchSubject.choose(Hue.GREEN));
        } catch (ExceptionInInitializerError error) {
            System.out.println(error.getClass().getSimpleName());
        }
    }
}
