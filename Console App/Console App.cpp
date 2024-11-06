#include <windows.h>
#include <iostream>

void simulateKeyPress(WORD vk) {
    INPUT input[2] = {};
    input[0].type = INPUT_KEYBOARD;
    input[0].ki.wVk = vk;
    input[1].type = INPUT_KEYBOARD;
    input[1].ki.wVk = vk;
    input[1].ki.dwFlags = KEYEVENTF_KEYUP;
    SendInput(2, input, sizeof(INPUT));
}

void playPause() {
    simulateKeyPress(VK_MEDIA_PLAY_PAUSE);
}

void nextTrack() {
    simulateKeyPress(VK_MEDIA_NEXT_TRACK);
}

void prevTrack() {
    simulateKeyPress(VK_MEDIA_PREV_TRACK);
}

int main() {
    RegisterHotKey(NULL, 1, MOD_NOREPEAT, VK_F5);
    RegisterHotKey(NULL, 2, MOD_NOREPEAT, VK_F6);
    RegisterHotKey(NULL, 3, MOD_NOREPEAT, VK_F7);

    MSG msg = { 0 };
    while (GetMessage(&msg, NULL, 0, 0)) {
        if (msg.message == WM_HOTKEY) {
            switch (msg.wParam) {
            case 1:
                playPause();
                break;
            case 2:
                nextTrack();
                break;
            case 3:
                prevTrack();
                break;
            }
        }
    }

    return 0;
}
