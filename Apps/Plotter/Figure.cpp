#include <QPainter>
#include <QThreadPool>
#include <QDebug>

#include "Figure.h"

Figure::Figure(relaptr_t relation, QWidget *parent)
    : QWidget(parent), relation(relation), d(0.01, 0.01), o(0, 0)
{
    resize(100, 100);
    for (int x = 0; x < width(); x++)
    {
        for (int y = 0; y < height(); y++)
        {
            QPointF p((x - o.x()) * d.x(), (y - o.y()) * d.y());
            if (!points_sol.contains(p) && !tasks_que.contains(p))
            {
                points_sol[p] = std::make_tuple(false, true);
                tasks_que.enqueue(p);
            }
        }
    }

    for (int i = 0; i < 1000; i++)
    {
        Consumer *t = new Consumer(relation, d, this);
        consumers.push_back(t);
        t->start();
    }

    timer = new QTimer(this);
    connect(timer, &QTimer::timeout, this, [&]()
            { update(); });
    timer->start(1000);
}

Figure::~Figure()
{
    for (auto t : consumers)
        t->terminate();
    if (!que_mutex.tryLock())
        que_mutex.unlock();
}

void Figure::paintEvent(QPaintEvent *)
{
    QPainter painter(this);
    for (int x = 0; x < width(); x++)
    {
        for (int y = 0; y < height(); y++)
        {
            QPointF p((x - o.x()) * d.x(), (y - o.y()) * d.y());
            if (!points_sol.contains(p) && !tasks_que.contains(p))
            {
                continue;
            }
            solution s = points_sol[p];
            bool a = std::get<0>(s);
            bool b = std::get<1>(s);
            if (a == false && b == false)
                painter.setBrush(QBrush(QColor(255, 255, 255)));
            else if (a == false && b == true)
                painter.setBrush(QBrush(QColor(255, 0, 0)));
            else if (a == true && b == true)
                painter.setBrush(QBrush(QColor(0, 0, 0)));
            painter.setPen(painter.brush().color());
            painter.drawPoint(x, y);
        }
    }
}