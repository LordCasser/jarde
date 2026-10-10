public class DeepFinally {
    static int trace;

    private static void cleanup() {
        trace++;
    }

    static int run(int x) {
        try {
            while (x > 33) {
                x--;
                while (x > 32) {
                    x--;
                    while (x > 31) {
                        x--;
                        while (x > 30) {
                            x--;
                            while (x > 29) {
                                x--;
                                while (x > 28) {
                                    x--;
                                    while (x > 27) {
                                        x--;
                                        while (x > 26) {
                                            x--;
                                            while (x > 25) {
                                                x--;
                                                while (x > 24) {
                                                    x--;
                                                    while (x > 23) {
                                                        x--;
                                                        while (x > 22) {
                                                            x--;
                                                            while (x > 21) {
                                                                x--;
                                                                while (x > 20) {
                                                                    x--;
                                                                    while (x > 19) {
                                                                        x--;
                                                                        while (x > 18) {
                                                                            x--;
                                                                            while (x > 17) {
                                                                                x--;
                                                                                while (x > 16) {
                                                                                    x--;
                                                                                    while (x > 15) {
                                                                                        x--;
                                                                                        while (x > 14) {
                                                                                            x--;
                                                                                            while (x > 13) {
                                                                                                x--;
                                                                                                while (x > 12) {
                                                                                                    x--;
                                                                                                    while (x > 11) {
                                                                                                        x--;
                                                                                                        while (x > 10) {
                                                                                                            x--;
                                                                                                            while (x > 9) {
                                                                                                                x--;
                                                                                                                while (x > 8) {
                                                                                                                    x--;
                                                                                                                    while (x > 7) {
                                                                                                                        x--;
                                                                                                                        while (x > 6) {
                                                                                                                            x--;
                                                                                                                            while (x > 5) {
                                                                                                                                x--;
                                                                                                                                while (x > 4) {
                                                                                                                                    x--;
                                                                                                                                    while (x > 3) {
                                                                                                                                        x--;
                                                                                                                                        while (x > 2) {
                                                                                                                                            x--;
                                                                                                                                            while (x > 1) {
                                                                                                                                                x--;
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                }
                                                                                                                            }
                                                                                                                        }
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return x;
        } finally {
            cleanup();
        }
    }
}
