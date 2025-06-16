#include <iostream>
#include <QDebug>

#include "Consumer.h"
#include "Interval.h"
#include "ExprSymbol.h"
#include "Float.h"
#include "Equality.h"
#include "EmptySet.h"

QHash<QPointF, solution> points_sol;
QQueue<QPointF> tasks_que;
QMutex que_mutex;

Consumer::Consumer(relaptr_t relation, QPointF d, QObject *parent)
    : relation(relation), d(d), QThread(parent)
{
}

void Consumer::run()
{
    while (true)
    {
        que_mutex.lock();
        if (tasks_que.isEmpty())
        {
            que_mutex.unlock();
            continue;
        }
        QPointF point = tasks_que.dequeue();
        que_mutex.unlock();

        double lx = point.x(), rx = point.x() + d.x();
        double ly = point.y(), ry = point.y() + d.y();

        exprptr_t x(new ExprSymbol("x",
                                   setptr_t(new Interval(exprptr_t(new Float(lx)),
                                                         exprptr_t(new Float(rx)),
                                                         true,
                                                         true))));
        exprptr_t y(new ExprSymbol("y",
                                   setptr_t(new Interval(exprptr_t(new Float(ly)),
                                                         exprptr_t(new Float(ry)),
                                                         true,
                                                         true))));
        setptr_t a = relation->lhs->replace(x, x)->replace(y, y)->belongto();
        setptr_t b = relation->rhs->replace(x, x)->replace(y, y)->belongto();

        solution s;
        if (isinstance<Equality>(relation))
        {
            if (isinstance<True>(a->operator==(b))) // T,T
                s = std::make_tuple(true, true);
            else if (isinstance<EmptySet>(a->operator&(b))) // F,F
                s = std::make_tuple(false, false);
            else // F,T
                s = std::make_tuple(false, true);
        }
        else
        {
            std::cout << "暂不支持的关系: " << typeid(*relation.get()).name() << "\n";
            return;
        }
        que_mutex.lock();
        points_sol[point] = s;
        que_mutex.unlock();
    }
}