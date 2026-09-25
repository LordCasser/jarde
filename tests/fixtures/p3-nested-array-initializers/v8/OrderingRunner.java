public final class OrderingRunner {
    public static void main(String[] args) {
        JadxOrderingControl.trace = 0;
        System.out.println(java.util.Arrays.toString(JadxOrderingControl.build())
            + ":" + JadxOrderingControl.trace);
    }
}
