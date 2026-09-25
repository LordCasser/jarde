package demo;

/* JADX INFO: loaded from: input-g.jar:demo/Runner.class */
public class Runner {
    public static void main(String[] args) {
        for (Op op : Op.values()) {
            System.out.println(op.tag() + "=" + op.apply(7, 3) + "/runtimeIsEnum=" + (op.getClass() == Op.class) + "/class=" + op.getClass().getName() + "/declaring=" + op.getDeclaringClass().getSimpleName());
        }
    }
}
