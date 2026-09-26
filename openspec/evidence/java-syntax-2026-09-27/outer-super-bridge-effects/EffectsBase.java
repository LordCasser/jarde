class EffectsBase {
    int combine(int left, int right) {
        OuterSuperEffects.events = OuterSuperEffects.events * 10 + 3;
        return left * 10 + right;
    }
}
