package defpackage;
public class StandDriver {
    public static void main(String[] args) {
        try {
            Runnable t=NoStand.make(null); System.out.println("creation=ok");
            try { t.run(); System.out.println("invocation=ok"); } catch (NullPointerException e) {System.out.println("invocation=NPE");}
        } catch (NullPointerException e) {System.out.println("creation=NPE");}
    }
}
