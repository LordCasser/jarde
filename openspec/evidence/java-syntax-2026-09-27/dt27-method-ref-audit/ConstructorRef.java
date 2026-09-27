package dt27;

class ConstructorRef {
    static Maker maker() {
        return RuntimeException::new;
    }
}
