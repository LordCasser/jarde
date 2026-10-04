package demo;

public class Probe {
    public static void main(String[] args) {
        System.out.println(LabeledOps.TIMES.name() + "=" + LabeledOps.TIMES.getLabel()
            + ":" + LabeledOps.TIMES.ordinal() + ":" + LabeledOps.TIMES.apply(2, 3)
            + ":" + LabeledOps.TIMES.getClass().getName());
        System.out.println(LabeledOps.DIVIDE.name() + "=" + LabeledOps.DIVIDE.getLabel()
            + ":" + LabeledOps.DIVIDE.ordinal() + ":" + LabeledOps.DIVIDE.apply(10, 5)
            + ":" + LabeledOps.DIVIDE.getClass().getName());
    }
}
