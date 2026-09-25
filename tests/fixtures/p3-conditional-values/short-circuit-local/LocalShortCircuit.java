class LocalShortCircuit {
    static String observed;
    static boolean visible;

    static void check(boolean gate) {
        String expected = "captured-value";
        visible = gate && expected.equals(observed);
    }
}
