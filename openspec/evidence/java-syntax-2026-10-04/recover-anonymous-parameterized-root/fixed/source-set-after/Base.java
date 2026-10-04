abstract class Base {
    Base() {
        AnonymousSuperDispatch.inBaseConstructor = true;
        observe();
        AnonymousSuperDispatch.capturedVisibleBeforeBaseReturns =
                AnonymousSuperDispatch.inBaseConstructor
                        && "captured-value".equals(AnonymousSuperDispatch.observed);
        AnonymousSuperDispatch.inBaseConstructor = false;
    }

    abstract void observe();
}

