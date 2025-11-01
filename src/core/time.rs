
use std::time::Instant;

// Константа для перевода наносекунд в миллисекунды
const NANOS_PER_MILLI: f32 = 1_000_000f32;

// Структура для отслеживания времени и FPS
pub struct TimeInfo {
    dt: f32,           // Время между кадрами в мс
    fps: f32,          // Текущий FPS
    frame_sum: f32,    // Сумма кадров для расчета FPS
    dt_sum: f32,       // Сумма времени для расчета FPS
    prev_time: Instant,  // Время предыдущего кадра
}

impl TimeInfo {
    // TimeInfo: проста утиліта для обчислення dt (мілісекунд між кадрами) та FPS.
    // Логіка:
    // - При кожному update() беремо Instant::now() і обчислюємо різницю в наносекундах (subsec_nanos()).
    // - dt зберігається в мілісекундах.
    // - Накопичуємо dt_sum та frame_sum; коли dt_sum >= 1000 ms обчислюємо fps = кадри / секунда.
    pub fn new() -> TimeInfo {
        TimeInfo { dt: 0.0, fps: 0.0, frame_sum: 0.0, dt_sum: 0.0, prev_time: Instant::now() }
    }

    #[allow(dead_code)]
    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.frame_sum += 1.0;
        // Assume duration is never over full second, so ignore whole seconds in Duration
        self.dt = now.duration_since(self.prev_time).subsec_nanos() as f32 / NANOS_PER_MILLI;
        self.dt_sum += self.dt;
        if self.dt_sum >= 1000.0 {
            self.fps = 1000.0 / (self.dt_sum / self.frame_sum);
            self.dt_sum = 0.0;
            self.frame_sum = 0.0;
        }
        self.prev_time = now;
    }
}
