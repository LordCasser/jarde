package demo;

public final class Probe {
    public static void main(String[] args) {
        for (Op op : Op.values()) {
            System.out.println(op.tag() + "=" + op.apply(7, 3));
        }
    }
}
