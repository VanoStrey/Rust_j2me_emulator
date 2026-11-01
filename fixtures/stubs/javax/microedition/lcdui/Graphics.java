package javax.microedition.lcdui;

public class Graphics {
    public static final int HCENTER = 1;
    public static final int VCENTER = 2;
    public static final int LEFT = 4;
    public static final int RIGHT = 8;
    public static final int TOP = 16;
    public static final int BOTTOM = 32;
    public static final int BASELINE = 64;

    public void setColor(int RGB) {}
    public void setColor(int red, int green, int blue) {}
    public int getColor() { return 0; }
    public void setGrayScale(int value) {}
    public int getGrayScale() { return 0; }
    public void fillRect(int x, int y, int width, int height) {}
    public void drawRect(int x, int y, int width, int height) {}
    public void drawLine(int x1, int y1, int x2, int y2) {}
    public void setClip(int x, int y, int width, int height) {}
    public void clipRect(int x, int y, int width, int height) {}
    public int getClipX() { return 0; }
    public int getClipY() { return 0; }
    public int getClipWidth() { return 0; }
    public int getClipHeight() { return 0; }
    public void translate(int x, int y) {}
    public int getTranslateX() { return 0; }
    public int getTranslateY() { return 0; }
    public void drawRGB(int[] rgbData, int offset, int scanlength, int x, int y, int width, int height, boolean processAlpha) {}
    public void fillTriangle(int x1, int y1, int x2, int y2, int x3, int y3) {}
    public void drawTriangle(int x1, int y1, int x2, int y2, int x3, int y3) {}
}
