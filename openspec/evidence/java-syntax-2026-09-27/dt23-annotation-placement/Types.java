package dt23;

@A(c = Types.class)
public class Types {
    @A(c = Types.class)
    public int field;

    @A(c = Types.class)
    public void first(@A(c = Types.class) int value) {
    }

    public void second(int ignored, @A(c = Types.class, i = 5) int value) {
    }
}
