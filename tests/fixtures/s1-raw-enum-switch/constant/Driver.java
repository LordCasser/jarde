package p;
public class Driver {
    public static void main(String[] args) {
        for (Hue hue : new Hue[] {Hue.𐐀RED, Hue.BLUE, Hue.GREEN}) {
            Subject.reset();
            System.out.println(Subject.choose(hue) + "|" + Subject.trace());
        }
        Subject.reset();
        try {
            Subject.choose(null);
            System.out.println("unexpected|" + Subject.trace());
        } catch (NullPointerException expected) {
            System.out.println("null|" + Subject.trace());
        }
    }
}
