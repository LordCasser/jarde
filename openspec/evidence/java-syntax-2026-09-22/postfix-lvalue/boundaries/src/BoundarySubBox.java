public final class BoundarySubBox extends BoundaryBaseBox {
    public int value;
    public int other;

    public BoundarySubBox(int base, int value, int other) {
        super(base);
        this.value = value;
        this.other = other;
    }
}
