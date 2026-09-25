package genericctornegative;

final class SameClassCall {
    public <T extends Number> SameClassCall(T value) {
    }

    static void create(Integer value) {
        new <Integer> SameClassCall(value);
    }
}
