package javax.microedition.lcdui.game;

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;

public abstract class GameCanvas extends Canvas {
    public static final int UP_PRESSED = 1 << 1;
    public static final int DOWN_PRESSED = 1 << 6;
    public static final int LEFT_PRESSED = 1 << 2;
    public static final int RIGHT_PRESSED = 1 << 5;
    public static final int FIRE_PRESSED = 1 << 8;
    public static final int GAME_A_PRESSED = 1 << 9;
    public static final int GAME_B_PRESSED = 1 << 10;
    public static final int GAME_C_PRESSED = 1 << 11;
    public static final int GAME_D_PRESSED = 1 << 12;

    protected GameCanvas(boolean suppressKeyEvents) {}
    public Graphics getGraphics() { return null; }
    public void flushGraphics() {}
    public void flushGraphics(int x, int y, int width, int height) {}
    public int getKeyStates() { return 0; }
    public void paint(Graphics g) {}
}
