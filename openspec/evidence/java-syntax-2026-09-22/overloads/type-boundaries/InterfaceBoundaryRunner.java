public class InterfaceBoundaryRunner {
    public static void main(String[] args) {
        System.out.println("object=" + InterfaceBoundaryProbe.caller(new Object()));
    }
}
