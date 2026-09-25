interface Child extends Parent {
    @Override default int value() { return 4; }
}
