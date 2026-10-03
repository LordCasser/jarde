public class E1 {
    enum Color { RED, GREEN, BLUE }
    enum Level { LOW, MID, HIGH }
    public static String byColor(Color c) {
        switch (c) {
            case RED: return "r";
            case GREEN: return "g";
            case BLUE: return "b";
            default: return "?";
        }
    }
    public static int byLevel(Level l) {
        switch (l) {
            case LOW: return 1;
            case MID: return 2;
            default: return 3;
        }
    }
    public static String twoSwitches(Color c, Level l) {
        String s;
        switch (c) { case RED: s = "r"; break; default: s = "x"; }
        switch (l) { case HIGH: s += "h"; break; default: s += "l"; }
        return s;
    }
    public static void main(String[] a) {
        System.out.println(byColor(Color.GREEN) + byLevel(Level.HIGH) + twoSwitches(Color.RED, Level.LOW));
        System.out.println(byColor(Color.BLUE) + byLevel(Level.MID) + twoSwitches(Color.BLUE, Level.HIGH));
    }
}
