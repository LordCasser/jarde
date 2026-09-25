public final class StaticQualifierRunner {
    public static void main(String[] args) {
        StaticQualifierProbe.reset();
        System.out.println("read=" + StaticQualifierProbe.read() + ":select=" + StaticQualifierProbe.selects + ":rhs=" + StaticQualifierProbe.rightCalls);

        StaticQualifierProbe.reset();
        System.out.println("write=" + StaticQualifierProbe.write() + ":select=" + StaticQualifierProbe.selects + ":rhs=" + StaticQualifierProbe.rightCalls);

        StaticQualifierProbe.reset();
        System.out.println("sum=" + StaticQualifierProbe.writeSum() + ":select=" + StaticQualifierProbe.selects + ":rhs=" + StaticQualifierProbe.rightCalls);

        StaticQualifierProbe.reset();
        StaticQualifierProbe.failSelect = true;
        try {
            StaticQualifierProbe.write();
            System.out.println("fail=returned");
        } catch (IllegalStateException expected) {
            System.out.println("fail=ISE:select=" + StaticQualifierProbe.selects + ":rhs=" + StaticQualifierProbe.rightCalls + ":value=" + StaticQualifierProbe.value);
        }
    }
}
