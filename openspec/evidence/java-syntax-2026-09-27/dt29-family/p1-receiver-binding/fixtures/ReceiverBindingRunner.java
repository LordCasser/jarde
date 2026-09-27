package dt29p1;

public class ReceiverBindingRunner {
    public static void main(String[] args) throws Exception {
        Class<?> family = Class.forName("dt29p1.ReceiverBindingFamily");
        Class<?> a = Class.forName("dt29p1.ReceiverBindingFamily$A");
        Class<?> b = Class.forName("dt29p1.ReceiverBindingFamily$B");
        Class<?> c = Class.forName("dt29p1.ReceiverBindingFamily$C");
        Class<?> d = Class.forName("dt29p1.ReceiverBindingFamily$D");
        Object receiver = b.newInstance();
        b.getMethod("self", boolean.class).invoke(receiver, true);
        boolean first = (Boolean) a.getMethod("all").invoke(receiver);
        c.getMethod("set", b, boolean.class).invoke(c.newInstance(), receiver, false);
        boolean second = (Boolean) a.getMethod("all").invoke(receiver);
        d.getMethod("set", b, boolean.class).invoke(d.newInstance(), receiver, true);
        boolean third = (Boolean) a.getMethod("all").invoke(receiver);
        System.out.println(family.getMethod("run").invoke(null) + ":" + first + ":" + second
                + ":" + third + ":" + b.getMethod("hidden").invoke(receiver));
    }
}
