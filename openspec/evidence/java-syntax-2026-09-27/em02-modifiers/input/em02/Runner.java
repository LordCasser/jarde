package em02;
public class Runner {
    public static void main(String[] args) {
        Contract c = new Contract() {
            public int value() { return 6; }
            public int alsoValue() { return 7; }
        };
        System.out.println(c.plusOne() + ":" + Contract.five());
        System.out.println(new PrivateBase().basePing() + ":" + new PrivateChild().ping() + ":" + new PrivateChild().basePing());
        System.out.println(new PackageChild().callBase() + ":" + new em02.other.CrossPackageChild().callBase());
    }
}
