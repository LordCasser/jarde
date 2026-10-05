public class DV {
    int a = 0; int b = 0;
    void orAcc(int x){ a |= x; b++; }                    // ior 版（FA.add 形）
    void xorAcc(int x){ a ^= x; }                        // ixor
    void andAcc(int x){ a &= x; }                        // iand
    void subAcc(int x){ a -= x; }                        // isub
    void mulAcc(int x){ a *= x; }                        // imul
    void divAcc(int x){ a /= x; }                        // idiv
    void shlAcc(int x){ a <<= x; }                       // ishl
}
