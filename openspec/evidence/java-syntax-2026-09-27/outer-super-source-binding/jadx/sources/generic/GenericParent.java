package generic;

/* JADX INFO: loaded from: GenericParent.class */
class GenericParent<T> {
    GenericParent() {
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    public String choose(T t) {
        return "type-variable";
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    public String choose(CharSequence charSequence) {
        return "char-sequence";
    }
}
