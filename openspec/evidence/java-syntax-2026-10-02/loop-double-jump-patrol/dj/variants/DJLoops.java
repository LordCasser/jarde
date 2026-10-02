public class DJLoops {
    public static String dblContLabels(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) continue;
                    if (j == 2) continue outer;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String dblBreakTargets(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (i == 2) break outer;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String brkContTwoLevels(int n) {
        StringBuilder b = new StringBuilder();
        outer:
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) break;
                    if (j == 2) continue outer;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String contSelfOnly(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++)
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (k == 1) continue;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String contMidLabelOnly(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++)
            mid:
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (j == 2) continue mid;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static String brkMidLabelOnly(int n) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < n; i++)
            mid:
            for (int j = 0; j < n; j++)
                for (int k = 0; k < n; k++) {
                    if (j == 2) break mid;
                    b.append(i).append(j).append(k).append(' ');
                }
        return b.toString();
    }
    public static void main(String[] a) {
        System.out.println(dblContLabels(3)); System.out.println(dblBreakTargets(3)); System.out.println(brkContTwoLevels(3));
        System.out.println(contSelfOnly(3)); System.out.println(contMidLabelOnly(3)); System.out.println(brkMidLabelOnly(3));
    }
}
