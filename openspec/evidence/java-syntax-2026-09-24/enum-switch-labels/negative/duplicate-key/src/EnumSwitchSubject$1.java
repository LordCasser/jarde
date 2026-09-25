final class EnumSwitchSubject$1 {
    static final int[] $SwitchMap$Hue;

    static {
        int[] map = new int[Hue.values().length];
        $SwitchMap$Hue = map;
        try {
            map[Hue.RED.ordinal()] = 1;
        } catch (NoSuchFieldError ignored) {
        }
        try {
            map[Hue.BLUE.ordinal()] = 1;
        } catch (NoSuchFieldError ignored) {
        }
    }
}
