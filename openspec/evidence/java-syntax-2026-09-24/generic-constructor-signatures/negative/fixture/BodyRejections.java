package genericctornegative;

final class BodyUsesParameter {
    private Number saved;

    public <T extends Number> BodyUsesParameter(T value) {
        this.saved = value;
    }
}

final class BodyUsesThis {
    public BodyUsesThis() {
    }

    public <T extends Number> BodyUsesThis(T value) {
        this();
    }
}
