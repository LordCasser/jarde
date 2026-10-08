public class Driver {
    public static void main(String[] args) {
        try {
            Thread t=NoCheck.make(null); System.out.println("creation=ok");
            try { t.run(); System.out.println("invocation=ok"); } catch (NullPointerException e) {System.out.println("invocation=NPE");}
        } catch (NullPointerException e) {System.out.println("creation=NPE");}
    }
}
