#pragma once

#include <QWidget>
#include <QPointF>
#include <QPaintEvent>
#include <QTimer>
#include <QVector>

#include "Consumer.h"

class Figure : public QWidget
{
public:
    Figure(relaptr_t, QWidget *parent = nullptr);
    ~Figure();

    void paintEvent(QPaintEvent *);

    QPointF d; // dx和dy
    QPoint o;  // 坐标原点在屏幕上的位置
    relaptr_t relation;
    QTimer *timer;
    QVector<Consumer *> consumers;
};