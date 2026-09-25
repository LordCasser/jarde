public final class NonInvokeQualifierRunner {
    public static void main(String[] args) {
        System.out.println("value=" + NonInvokeQualifierProbe.call(null));
    }
}
