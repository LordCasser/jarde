package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11HandlerWriteControl.class */
public final class CF11HandlerWriteControl {
    private static Object asObject(Object obj) {
        return obj;
    }

    public static Object handlerWrite() {
        try {
            throw new IllegalArgumentException("handler");
        } catch (IllegalArgumentException e) {
            return asObject(e);
        }
    }
}
