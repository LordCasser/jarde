public final class Runner {
    public static void main(String[] args) {
        CarryDependencyProbe receiver = new CarryDependencyProbe();
        for (boolean flag : new boolean[] {false, true}) {
            CarryDependencyProbe.effects = 0;
            CarryDependencyProbe.trace = "";
            CarryDependencyProbe.referenceConsumer(flag);
            System.out.println(flag + "|" + CarryDependencyProbe.effects + "|" + CarryDependencyProbe.trace);
            CarryDependencyProbe.effects = 0;
            CarryDependencyProbe.trace = "";
            receiver.thisConsumer(flag);
            System.out.println("this|" + flag + "|" + CarryDependencyProbe.effects + "|" + CarryDependencyProbe.trace);
            CarryDependencyProbe.effects = 0;
            CarryDependencyProbe.trace = "";
            CarryDependencyProbe.parameterConsumer(receiver, flag);
            System.out.println("parameter|" + flag + "|" + CarryDependencyProbe.effects + "|" + CarryDependencyProbe.trace);
        }
    }
}
