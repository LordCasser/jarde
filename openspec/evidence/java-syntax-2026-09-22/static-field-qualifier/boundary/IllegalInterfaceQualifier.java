final class IllegalInterfaceQualifier {
    static InterfaceStaticOwner receiver() {
        return null;
    }

    static int call() {
        return receiver().value();
    }
}
