abstract class Base {
    Base() {
        Probe.flag = true;
        observe();
        Probe.other = Probe.other2 = Probe.flag && "captured-value".equals(Probe.text);
        Probe.flag = false;
    }
    abstract void observe();
}
class Probe {
    static boolean flag, other, other2;
    static String text;
}
