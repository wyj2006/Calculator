#include <QApplication>
#include <iostream>

#include "Figure.h"
#include "Relation.h"
#include "Equality.h"
#include "Runtime.h"
#include "Consumer.h"
#include "ExprSymbol.h"

class Plot : public Function
{
public:
    Plot(relaptr_t);

    virtual objptr_t _simplify_()
    {
        return Function::_simplify_();
    }

    relaptr_t relation;
};

class PlotMapping : public Mapping
{
public:
    PlotMapping() : Mapping("plot") {}

    virtual objptr_t operator()(funcargs_t args)
    {
        if (args.size() != 1 || !isinstance<Relation>(args[0]))
            throw std::runtime_error("[PlotMapping]超出定义域");
        return objptr_t(new Plot(relaptr_t(dynamic_cast<Relation *>(args[0]->copyThis()))));
    }
};

Plot::Plot(relaptr_t relation) : relation(relation), Function("plot", {relation})
{
    this->mapping = mappingptr_t(new PlotMapping());
}

#ifdef __cplusplus
extern "C"
{
#endif

#ifdef _WIN32
    namespace Win
    {
#include "windows.h"
        void WINAPI load(Runtime *runtime)
        {
            runtime->defName("plot", objptr_t(new PlotMapping()));
        }
    };
    using Win::load;
#endif

#ifdef __cplusplus
}
#endif

int main(int argc, char **argv)
{
    QApplication app(argc, argv);

    exprptr_t x(new ExprSymbol("x"));
    exprptr_t y(new ExprSymbol("y"));

    Figure w(relaptr_t(new Equality(y, x->pow(Integer(2)))));

    w.show();

    return app.exec();
}