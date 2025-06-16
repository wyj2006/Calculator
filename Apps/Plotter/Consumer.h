#pragma once

#include <QObject>
#include <QPointF>
#include <tuple>
#include <QHash>
#include <QQueue>
#include <QThread>
#include <QMutex>

#include "Relation.h"

typedef std::shared_ptr<Relation> relaptr_t;

class Consumer : public QThread
{
public:
    Consumer(relaptr_t, QPointF, QObject *);

    void run();
    QPointF d;
    relaptr_t relation;
};

static uint qHash(const QPointF &key, uint seed)
{
    return qHash<QString>(QString::number(key.x()) + QString::number(key.y()), seed);
}

typedef std::tuple<bool, bool> solution;
extern QHash<QPointF, solution> points_sol;
extern QQueue<QPointF> tasks_que;
extern QMutex que_mutex;